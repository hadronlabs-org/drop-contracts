use cosmos_sdk_proto::cosmos::{
    base::{abci::v1beta1::TxMsgData, v1beta1::Coin},
    staking::v1beta1::{
        MsgBeginRedelegateResponse, MsgDelegate, MsgDelegateResponse, MsgUndelegate,
        MsgUndelegateResponse,
    },
};
use cosmwasm_std::{entry_point, to_json_vec, CosmosMsg, Deps, Reply, StdError, SubMsg, Uint128};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
        types::ProtobufAny,
    },
    interchain_queries::v045::new_register_delegator_delegations_query_msg,
    interchain_txs::helpers::{decode_message_response, get_port_id},
    sudo::msg::{RequestPacket, SudoMsg},
    NeutronError, NeutronResult,
};

use lido_interchain_interceptor_base::{
    error::ContractResult,
    msg::{QueryMsg, SudoPayload},
    state::{InterchainIntercaptorBase, State, ICA_ID, SUDO_PAYLOAD_REPLY_ID},
};
use prost::Message;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, Transaction},
    proto::cosmos::base::v1beta1::Coin as ProtoCoin,
    proto::liquidstaking::staking::v1beta1::{
        MsgBeginRedelegate, MsgRedeemTokensforShares as MsgRedeemTokensForShares,
        MsgTokenizeShares, MsgTokenizeSharesResponse,
    },
    state::Config,
};

pub type InterchainInterceptor<'a> = InterchainIntercaptorBase<'a, Config, Transaction>;

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

    InterchainInterceptor::default().instantiate(deps, config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    InterchainInterceptor::default().query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();

    match msg {
        ExecuteMsg::Delegate {
            validator,
            amount,
            timeout,
        } => execute_delegate(deps, env, info, validator, amount, timeout),
        ExecuteMsg::Undelegate {
            validator,
            amount,
            timeout,
        } => execute_undelegate(deps, env, info, validator, amount, timeout),
        ExecuteMsg::Redelegate {
            validator_from,
            validator_to,
            amount,
            timeout,
        } => execute_redelegate(
            deps,
            env,
            info,
            validator_from,
            validator_to,
            amount,
            timeout,
        ),
        ExecuteMsg::TokenizeShare {
            validator,
            amount,
            timeout,
        } => execute_tokenize_share(deps, env, info, validator, amount, timeout),
        ExecuteMsg::RedeemShare {
            validator,
            amount,
            denom,
            timeout,
        } => execute_redeem_share(deps, env, info, validator, amount, denom, timeout),
        ExecuteMsg::RegisterDelegatorDelegationsQuery { validators } => {
            register_delegations_query(deps, validators)
        }
        _ => interceptor_base.execute(deps, env, msg.to_base_enum()),
    }
}

fn register_delegations_query(
    deps: DepsMut<NeutronQuery>,
    validators: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();
    let config = interceptor_base.config.load(deps.storage)?;
    let state: State = interceptor_base.state.load(deps.storage)?;

    let delegator = state.ica.ok_or_else(|| {
        StdError::generic_err("Interchain account is not registered. Please register it first")
    })?;
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
    env: Env,
    _info: MessageInfo,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();
    let config: Config = interceptor_base.config.load(deps.storage)?;
    let state: State = interceptor_base.state.load(deps.storage)?;
    let delegator = state.ica.ok_or_else(|| {
        StdError::generic_err("Interchain account is not registered. Please register it first")
    })?;
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
        env,
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
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_undelegate(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();
    let config: Config = interceptor_base.config.load(deps.storage)?;
    let state: State = interceptor_base.state.load(deps.storage)?;

    let delegator = state.ica.ok_or_else(|| {
        StdError::generic_err("Interchain account is not registered. Please register it first")
    })?;

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
        env,
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
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_redelegate(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    validator_from: String,
    validator_to: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();
    let config: Config = interceptor_base.config.load(deps.storage)?;
    let state: State = interceptor_base.state.load(deps.storage)?;

    let delegator = state.ica.ok_or_else(|| {
        StdError::generic_err("Interchain account is not registered. Please register it first")
    })?;
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
        env,
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
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_tokenize_share(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();
    let config: Config = interceptor_base.config.load(deps.storage)?;
    let state: State = interceptor_base.state.load(deps.storage)?;

    let delegator = state.ica.ok_or_else(|| {
        StdError::generic_err("Interchain account is not registered. Please register it first")
    })?;
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
        env,
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
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_redeem_share(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    validator: String,
    amount: Uint128,
    denom: String,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();
    let config: Config = interceptor_base.config.load(deps.storage)?;
    let state: State = interceptor_base.state.load(deps.storage)?;

    let delegator = state.ica.ok_or_else(|| {
        StdError::generic_err("Interchain account is not registered. Please register it first")
    })?;
    let redeem_msg = MsgRedeemTokensForShares {
        delegator_address: delegator,
        amount: Some(ProtoCoin {
            denom: denom.to_string(),
            amount: amount.to_string(),
        }),
    };

    let submsg = compose_submsg(
        deps.branch(),
        env,
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
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn compose_submsg<T: prost::Message>(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    config: Config,
    in_msg: T,
    type_url: String,
    sudo_payload: Transaction,
    timeout: Option<u64>,
) -> NeutronResult<SubMsg<NeutronMsg>> {
    let interceptor_base = InterchainInterceptor::default();
    let ibc_fee: IbcFee = interceptor_base.ibc_fee.load(deps.storage)?;
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

    let submsg = msg_with_sudo_callback(
        deps.branch(),
        cosmos_msg,
        SudoPayload {
            port_id: get_port_id(env.contract.address.as_str(), ICA_ID),
            message: "message".to_string(),
            info: Some(sudo_payload),
        },
    )?;
    Ok(submsg)
}

fn msg_with_sudo_callback<C: Into<CosmosMsg<T>>, T>(
    deps: DepsMut<NeutronQuery>,
    msg: C,
    payload: SudoPayload<Transaction>,
) -> StdResult<SubMsg<T>> {
    let interceptor_base = InterchainInterceptor::default();
    interceptor_base
        .reply_id_storage
        .save(deps.storage, &to_json_vec(&payload)?)?;

    Ok(SubMsg::reply_on_success(msg, SUDO_PAYLOAD_REPLY_ID))
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    let interceptor_base = InterchainInterceptor::default();

    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?} block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::TxQueryResult {
            query_id,
            height,
            data,
        } => interceptor_base.sudo_tx_query_result(deps, env, query_id, height, data),
        SudoMsg::KVQueryResult { query_id } => {
            interceptor_base.sudo_kv_query_result(deps, env, query_id)
        }
        SudoMsg::OpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => interceptor_base.sudo_open_ack(
            deps,
            env,
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        ),
        _ => Ok(Response::default()),
    }
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    data: Binary,
) -> NeutronResult<Response> {
    let interceptor_base = InterchainInterceptor::default();

    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let channel_id = request
        .source_channel
        .ok_or_else(|| StdError::generic_err("channel_id not found"))?;

    let payload = interceptor_base
        .sudo_payload
        .load(deps.storage, (channel_id.clone(), seq_id))?;
    deps.api
        .debug(&format!("WASMDEBUG: sudo_response: data: {data:?}"));

    let msg_data: TxMsgData = TxMsgData::decode(data.as_slice())?;
    deps.api
        .debug(&format!("WASMDEBUG: msg_data: data: {msg_data:?}"));

    #[allow(deprecated)]
    for item in msg_data.data {
        deps.api.debug(&format!("WASMDEBUG: item: data: {item:?}"));

        match item.msg_type.as_str() {
            "/cosmos.staking.v1beta1.MsgDelegate" => {
                deps.api
                    .debug("WASMDEBUG: sudo_response: MsgDelegateResponse");
                let out: MsgDelegateResponse = decode_message_response(&item.data)?;
                deps.api.debug(&format!(
                    "WASMDEBUG: sudo_response: MsgDelegateResponse: {out:?}"
                ));
                let mut txs = interceptor_base.transactions.load(deps.storage)?;
                txs.extend(payload.info.clone());
                interceptor_base.transactions.save(deps.storage, &txs)?;
                interceptor_base
                    .sudo_payload
                    .remove(deps.storage, (channel_id.clone(), seq_id));
            }
            "/cosmos.staking.v1beta1.MsgUndelegate" => {
                deps.api
                    .debug("WASMDEBUG: sudo_response: MsgUndelegateResponse");
                let out: MsgUndelegateResponse = decode_message_response(&item.data)?;
                deps.api.debug(&format!(
                    "WASMDEBUG: sudo_response: MsgUndelegateResponse: {out:?}"
                ));
                let mut txs = interceptor_base.transactions.load(deps.storage)?;
                txs.extend(payload.info.clone());
                interceptor_base.transactions.save(deps.storage, &txs)?;
                interceptor_base
                    .sudo_payload
                    .remove(deps.storage, (channel_id.clone(), seq_id));
            }
            "/cosmos.staking.v1beta1.MsgTokenizeShares" => {
                deps.api
                    .debug("WASMDEBUG: sudo_response: MsgTokenizeSharesResponse");
                let out: MsgTokenizeSharesResponse = decode_message_response(&item.data)?;
                deps.api.debug(&format!(
                    "WASMDEBUG: sudo_response: MsgTokenizeSharesResponse: {out:?}"
                ));
                let denom = out.amount.map(|coin| coin.denom).unwrap_or_default();
                let mut txs = interceptor_base.transactions.load(deps.storage)?;
                let info = match payload.info.clone() {
                    Some(info) => match info {
                        Transaction::TokenizeShare {
                            interchain_account_id,
                            validator,
                            denom: _,
                            amount,
                        } => Some(Transaction::TokenizeShare {
                            interchain_account_id,
                            validator,
                            denom,
                            amount,
                        }),
                        _ => Some(info),
                    },
                    None => None,
                };
                txs.extend(info);
                interceptor_base.transactions.save(deps.storage, &txs)?;
                interceptor_base
                    .sudo_payload
                    .remove(deps.storage, (channel_id.clone(), seq_id));
            }
            "/cosmos.staking.v1beta1.MsgBeginRedelegate" => {
                deps.api
                    .debug("WASMDEBUG: sudo_response: MsgBeginRedelegateResponse");
                let out: MsgBeginRedelegateResponse = decode_message_response(&item.data)?;
                deps.api.debug(&format!(
                    "WASMDEBUG: sudo_response: MsgBeginRedelegateResponse: {out:?}"
                ));
                let mut txs = interceptor_base.transactions.load(deps.storage)?;
                txs.extend(payload.info.clone());
                interceptor_base.transactions.save(deps.storage, &txs)?;
                interceptor_base
                    .sudo_payload
                    .remove(deps.storage, (channel_id.clone(), seq_id));
            }
            "/cosmos.staking.v1beta1.MsgRedeemTokensForShares" => {
                deps.api
                    .debug("WASMDEBUG: sudo_response: MsgRedeemTokensForSharesResponse");
                let out: MsgRedeemTokensForShares = decode_message_response(&item.data)?;
                deps.api.debug(&format!(
                    "WASMDEBUG: sudo_response: MsgRedeemTokensForSharesResponse: {out:?}"
                ));
                let denom = out.amount.map(|coin| coin.denom).unwrap_or_default();
                let mut txs = interceptor_base.transactions.load(deps.storage)?;
                let info: Option<Transaction> = match payload.info.clone() {
                    Some(info) => match info {
                        Transaction::TokenizeShare {
                            interchain_account_id,
                            validator,
                            denom: _,
                            amount,
                        } => Some(Transaction::RedeemShare {
                            interchain_account_id,
                            validator,
                            denom,
                            amount,
                        }),
                        _ => Some(info),
                    },
                    None => None,
                };
                txs.extend(info);
                interceptor_base.transactions.save(deps.storage, &txs)?;
                interceptor_base
                    .sudo_payload
                    .remove(deps.storage, (channel_id.clone(), seq_id));
            }
            _ => {
                deps.api.debug(
                    format!("This type of acknowledgement is not implemented: {payload:?}")
                        .as_str(),
                );
            }
        }
    }
    Ok(Response::default())
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    InterchainInterceptor::default().reply(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
