use std::{str::FromStr, vec};

use cosmos_sdk_proto::cosmos::{
    base::{abci::v1beta1::TxMsgData, v1beta1::Coin},
    staking::v1beta1::{MsgDelegate, MsgUndelegate},
};
use cosmwasm_std::{
    attr, ensure_eq, entry_point, to_json_binary, CosmosMsg, Deps, Reply, StdError, SubMsg,
    Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_staking_base::helpers::answer::response;
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
        types::ProtobufAny,
    },
    interchain_queries::v045::new_register_delegator_delegations_query_msg,
    interchain_txs::helpers::decode_message_response,
    sudo::msg::{RequestPacket, SudoMsg},
    NeutronError, NeutronResult,
};

use lido_puppeteer_base::{
    error::ContractResult,
    msg::{QueryMsg, ResponseHookErrorMsg, ResponseHookMsg, ResponseHookSuccessMsg, Transaction},
    state::{IcaState, PuppeteerBase, State, TxState, TxStateStatus, ICA_ID},
};

use prost::Message;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg},
    proto::cosmos::base::v1beta1::Coin as ProtoCoin,
    proto::liquidstaking::staking::v1beta1::{
        MsgBeginRedelegate, MsgBeginRedelegateResponse, MsgDelegateResponse,
        MsgRedeemTokensforShares, MsgRedeemTokensforSharesResponse, MsgTokenizeShares,
        MsgTokenizeSharesResponse, MsgUndelegateResponse,
    },
    state::Config,
};

pub type Puppeteer<'a> = PuppeteerBase<'a, Config>;

const CONTRACT_NAME: &str = concat!("crates.io:lido-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    let config = &Config {
        connection_id: msg.connection_id,
        port_id: msg.port_id,
        update_period: msg.update_period,
        remote_denom: msg.remote_denom,
        owner,
    };

    Puppeteer::default().instantiate(deps, config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Puppeteer::default().query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();

    match msg {
        ExecuteMsg::Delegate {
            validator,
            amount,
            timeout,
            reply_to,
        } => execute_delegate(deps, env, validator, amount, timeout, reply_to),
        ExecuteMsg::Undelegate {
            validator,
            amount,
            timeout,
            reply_to,
        } => execute_undelegate(deps, env, validator, amount, timeout, reply_to),
        ExecuteMsg::Redelegate {
            validator_from,
            validator_to,
            amount,
            timeout,
            reply_to,
        } => execute_redelegate(
            deps,
            env,
            validator_from,
            validator_to,
            amount,
            timeout,
            reply_to,
        ),
        ExecuteMsg::TokenizeShare {
            validator,
            amount,
            timeout,
            reply_to,
        } => execute_tokenize_share(deps, env, validator, amount, timeout, reply_to),
        ExecuteMsg::RedeemShare {
            validator,
            amount,
            denom,
            timeout,
            reply_to,
        } => execute_redeem_share(deps, env, validator, amount, denom, timeout, reply_to),
        ExecuteMsg::RegisterDelegatorDelegationsQuery { validators } => {
            register_delegations_query(deps, validators)
        }
        _ => puppeteer_base.execute(deps, env, msg.to_base_enum()),
    }
}

fn register_delegations_query(
    deps: DepsMut<NeutronQuery>,
    validators: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.storage)?;
    let delegator = puppeteer_base.get_ica(&puppeteer_base.state.load(deps.storage)?)?;
    let msg = new_register_delegator_delegations_query_msg(
        config.connection_id,
        delegator,
        validators,
        config.update_period,
    )?;
    Ok(Response::new().add_message(msg))
}

fn execute_delegate(
    mut deps: DepsMut<NeutronQuery>,
    _env: Env,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    let delegator = puppeteer_base.get_ica(&puppeteer_base.state.load(deps.storage)?)?;
    let delegate_msg = MsgDelegate {
        delegator_address: delegator,
        validator_address: validator.to_string(),
        amount: Some(Coin {
            denom: config.remote_denom.to_string(),
            amount: amount.to_string(),
        }),
    };

    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        delegate_msg,
        "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        Transaction::Delegate {
            interchain_account_id: ICA_ID.to_string(),
            validator,
            denom: config.remote_denom,
            amount: amount.into(),
        },
        timeout,
        reply_to,
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_undelegate(
    mut deps: DepsMut<NeutronQuery>,
    _env: Env,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    let delegator = puppeteer_base.get_ica(&puppeteer_base.state.load(deps.storage)?)?;

    let undelegate_msg = MsgUndelegate {
        delegator_address: delegator,
        validator_address: validator.to_string(),
        amount: Some(Coin {
            denom: config.remote_denom.to_string(),
            amount: amount.to_string(),
        }),
    };

    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        undelegate_msg,
        "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
        Transaction::Undelegate {
            interchain_account_id: ICA_ID.to_string(),
            validator,
            denom: config.remote_denom,
            amount: amount.into(),
        },
        timeout,
        reply_to,
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_redelegate(
    mut deps: DepsMut<NeutronQuery>,
    _env: Env,
    validator_from: String,
    validator_to: String,
    amount: Uint128,
    timeout: Option<u64>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    let delegator = puppeteer_base.get_ica(&puppeteer_base.state.load(deps.storage)?)?;
    let redelegate_msg = MsgBeginRedelegate {
        delegator_address: delegator,
        validator_src_address: validator_from.to_string(),
        validator_dst_address: validator_to.to_string(),
        amount: Some(ProtoCoin {
            denom: config.remote_denom.to_string(),
            amount: amount.to_string(),
        }),
    };

    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        redelegate_msg,
        "/cosmos.staking.v1beta1.MsgBeginRedelegate".to_string(),
        Transaction::Redelegate {
            interchain_account_id: ICA_ID.to_string(),
            validator_from,
            validator_to,
            denom: config.remote_denom,
            amount: amount.into(),
        },
        timeout,
        reply_to,
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_tokenize_share(
    mut deps: DepsMut<NeutronQuery>,
    _env: Env,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    let delegator = puppeteer_base.get_ica(&puppeteer_base.state.load(deps.storage)?)?;
    let tokenize_msg = MsgTokenizeShares {
        delegator_address: delegator.clone(),
        validator_address: validator.to_string(),
        tokenized_share_owner: delegator,
        amount: Some(ProtoCoin {
            denom: config.remote_denom.to_string(),
            amount: amount.to_string(),
        }),
    };
    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        tokenize_msg,
        "/cosmos.staking.v1beta1.MsgTokenizeShares".to_string(),
        Transaction::TokenizeShare {
            interchain_account_id: ICA_ID.to_string(),
            validator,
            denom: config.remote_denom,
            amount: amount.into(),
        },
        timeout,
        reply_to,
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_redeem_share(
    mut deps: DepsMut<NeutronQuery>,
    _env: Env,
    validator: String,
    amount: Uint128,
    denom: String,
    timeout: Option<u64>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![
        attr("action", "redeem_share"),
        attr("validator", validator.clone()),
        attr("amount", amount.to_string()),
        attr("denom", denom.clone()),
    ];
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    let delegator = puppeteer_base.get_ica(&puppeteer_base.state.load(deps.storage)?)?;
    let redeem_msg = MsgRedeemTokensforShares {
        delegator_address: delegator,
        amount: Some(ProtoCoin {
            denom: denom.to_string(),
            amount: amount.to_string(),
        }),
    };
    let submsg = compose_submsg(
        deps.branch(),
        config,
        redeem_msg,
        "/cosmos.staking.v1beta1.MsgRedeemTokensForShares".to_string(),
        Transaction::RedeemShare {
            interchain_account_id: ICA_ID.to_string(),
            validator,
            denom,
            amount: amount.into(),
        },
        timeout,
        reply_to,
    )?;
    Ok(Response::default()
        .add_submessages(vec![submsg])
        .add_attributes(attrs))
}

fn compose_submsg<T: prost::Message>(
    mut deps: DepsMut<NeutronQuery>,
    config: Config,
    in_msg: T,
    type_url: String,
    transaction: Transaction,
    timeout: Option<u64>,
    reply_to: String,
) -> NeutronResult<SubMsg<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let ibc_fee: IbcFee = puppeteer_base.ibc_fee.load(deps.storage)?;
    let connection_id = config.connection_id;
    let mut buf = Vec::new();
    buf.reserve(in_msg.encoded_len());

    if let Err(e) = in_msg.encode(&mut buf) {
        return Err(NeutronError::Std(StdError::generic_err(format!(
            "Encode error: {e}"
        ))));
    }
    let any_msg = ProtobufAny {
        type_url,
        value: Binary::from(buf),
    };
    let cosmos_msg = NeutronMsg::submit_tx(
        connection_id,
        ICA_ID.to_string(),
        vec![any_msg],
        "".to_string(),
        timeout.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
        ibc_fee,
    );
    let submsg =
        puppeteer_base.msg_with_sudo_callback(deps.branch(), cosmos_msg, transaction, reply_to)?;
    Ok(submsg)
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    let puppeteer_base = Puppeteer::default();
    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?} block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_timeout(deps, env, request),
        SudoMsg::TxQueryResult {
            query_id,
            height,
            data,
        } => puppeteer_base.sudo_tx_query_result(deps, env, query_id, height, data),
        SudoMsg::KVQueryResult { query_id } => {
            puppeteer_base.sudo_kv_query_result(deps, env, query_id)
        }
        SudoMsg::OpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => puppeteer_base.sudo_open_ack(
            deps,
            env,
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        ),
    }
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    data: Binary,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let puppeteer_base = Puppeteer::default();
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let tx_state = puppeteer_base.tx_state.load(deps.storage)?;
    if tx_state.status != TxStateStatus::InProgress {
        return Err(NeutronError::Std(StdError::generic_err(
            "Transaction state is not in progress",
        )));
    }
    let reply_to = tx_state
        .reply_to
        .ok_or_else(|| StdError::generic_err("reply_to not found"))?;

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    puppeteer_base.tx_state.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::Idle,
            seq_id: None,
            transaction: None,
            reply_to: None,
        },
    )?;

    let msg_data: TxMsgData = TxMsgData::decode(data.as_slice())?;
    deps.api
        .debug(&format!("WASMDEBUG: msg_data: data: {msg_data:?}"));

    let mut msgs = vec![];
    #[allow(deprecated)]
    for item in msg_data.data {
        let answer = match item.msg_type.as_str() {
            "/cosmos.staking.v1beta1.MsgDelegate" => {
                let _out: MsgDelegateResponse = decode_message_response(&item.data)?;
                lido_puppeteer_base::msg::ResponseAnswer::DelegateResponse(
                    lido_puppeteer_base::proto::MsgDelegateResponse {},
                )
            }
            "/cosmos.staking.v1beta1.MsgUndelegate" => {
                let out: MsgUndelegateResponse = decode_message_response(&item.data)?;
                lido_puppeteer_base::msg::ResponseAnswer::UndelegateResponse(
                    lido_puppeteer_base::proto::MsgUndelegateResponse {
                        completion_time: out.completion_time.map(|t| t.into()),
                    },
                )
            }
            "/cosmos.staking.v1beta1.MsgTokenizeShares" => {
                let out: MsgTokenizeSharesResponse = decode_message_response(&item.data)?;
                lido_puppeteer_base::msg::ResponseAnswer::TokenizeSharesResponse(
                    lido_puppeteer_base::proto::MsgTokenizeSharesResponse {
                        amount: out.amount.map(convert_coin).transpose()?,
                    },
                )
            }
            "/cosmos.staking.v1beta1.MsgBeginRedelegate" => {
                let out: MsgBeginRedelegateResponse = decode_message_response(&item.data)?;
                lido_puppeteer_base::msg::ResponseAnswer::BeginRedelegateResponse(
                    lido_puppeteer_base::proto::MsgBeginRedelegateResponse {
                        completion_time: out.completion_time.map(|t| t.into()),
                    },
                )
            }
            "/cosmos.staking.v1beta1.MsgRedeemTokensForShares" => {
                let out: MsgRedeemTokensforSharesResponse = decode_message_response(&item.data)?;
                lido_puppeteer_base::msg::ResponseAnswer::RedeemTokensforSharesResponse(
                    lido_puppeteer_base::proto::MsgRedeemTokensforSharesResponse {
                        amount: out.amount.map(convert_coin).transpose()?,
                    },
                )
            }
            _ => {
                deps.api.debug(
                    format!("This type of acknowledgement is not implemented: {item:?}").as_str(),
                );
                lido_puppeteer_base::msg::ResponseAnswer::UnknownResponse {}
            }
        };
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to.clone(),
            msg: to_json_binary(&ResponseHookMsg::Success(ResponseHookSuccessMsg {
                request_id: seq_id,
                request: request.clone(),
                transaction: transaction.clone(),
                answer,
            }))?,
            funds: vec![],
        }))
    }
    Ok(response("sudo-response", "puppeteer", attrs).add_messages(msgs))
}

fn convert_coin(coin: crate::proto::cosmos::base::v1beta1::Coin) -> StdResult<cosmwasm_std::Coin> {
    Ok(cosmwasm_std::Coin {
        denom: coin.denom,
        amount: Uint128::from_str(&coin.amount)?,
    })
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    details: String,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("action", "sudo_error"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
        attr("details", details.clone()),
    ];
    let puppeteer_base: PuppeteerBase<'_, Config> = Puppeteer::default();
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_error: request: {request:?} details: {details:?}",
        request = request,
        details = details
    ));
    let tx_state = puppeteer_base.tx_state.load(deps.storage)?;
    ensure_eq!(
        tx_state.status,
        TxStateStatus::InProgress,
        NeutronError::Std(StdError::generic_err(
            "Transaction state is not in progress",
        ))
    );
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tx_state
            .reply_to
            .ok_or_else(|| StdError::generic_err("reply_to not found"))?,
        msg: to_json_binary(&ResponseHookMsg::Error(ResponseHookErrorMsg {
            request_id: seq_id,
            request,
            details,
        }))?,
        funds: vec![],
    });
    puppeteer_base.tx_state.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::Idle,
            seq_id: None,
            transaction: None,
            reply_to: None,
        },
    )?;
    Ok(response("sudo-error", "puppeteer", attrs).add_message(msg))
}

fn sudo_timeout(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("action", "sudo_timeout"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let puppeteer_base: PuppeteerBase<'_, Config> = Puppeteer::default();
    puppeteer_base.state.save(
        deps.storage,
        &State {
            ica: None,
            last_processed_height: None,
            ica_state: IcaState::Timeout,
        },
    )?;
    puppeteer_base.tx_state.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::Idle,
            seq_id: None,
            transaction: None,
            reply_to: None,
        },
    )?;
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_timeout: request: {request:?}",
        request = request
    ));
    Ok(response("sudo-timeout", "puppeteer", attrs))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    Puppeteer::default().reply(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
