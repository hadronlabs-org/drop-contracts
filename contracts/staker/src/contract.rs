use cosmos_sdk_proto::cosmos::authz::v1beta1::MsgExec;
use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData;
use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    attr, coin, ensure, entry_point, to_json_binary, Coin, CosmosMsg, Deps, Reply, StdError,
    SubMsg, Uint128,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_utils::must_pay;
use drop_helpers::answer::response;
use drop_helpers::interchain_tx::prepare_any_msg;
use drop_staking_base::msg::staker::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, OpenAckVersion, QueryMsg,
};
use drop_staking_base::state::staker::{
    Config, ConfigOptional, ReplyMsg, Transaction, TxState, TxStateStatus, CONFIG, ICA, ICA_ID,
    NON_STAKED_BALANCE, TX_STATE,
};
use neutron_sdk::bindings::msg::{IbcFee, MsgIbcTransferResponse, MsgSubmitTxResponse, NeutronMsg};
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, SudoMsg};
use neutron_sdk::{NeutronError, NeutronResult};
use prost::Message;

use crate::error::{ContractError, ContractResult};

const CONTRACT_NAME: &str = concat!("crates.io:drop-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    let attrs = vec![
        attr("contract_name", CONTRACT_NAME),
        attr("contract_version", CONTRACT_VERSION),
        attr("msg", format!("{:?}", msg)),
        attr("sender", &info.sender),
    ];
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.owner.unwrap_or(info.sender.to_string()).as_str()),
    )?;
    TX_STATE.save(deps.storage, &TxState::default())?;
    CONFIG.save(
        deps.storage,
        &Config {
            port_id: msg.port_id,
            transfer_channel_id: msg.transfer_channel_id,
            connection_id: msg.connection_id,
            ibc_fees: msg.ibc_fees,
            timeout: msg.timeout,
            local_denom: msg.local_denom,
            remote_denom: msg.remote_denom,
            base_denom: msg.base_denom,
            allowed_addresses: msg.allowed_addresses,
            puppeteer_ica: None,
        },
    )?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Ica {} => query_ica(deps),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            to_json_binary(&ownership).map_err(NeutronError::Std)
        }
    }
}

fn query_non_staked_balance(deps: Deps, env: Env) -> NeutronResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let local_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), config.base_denom)?
        .amount;
    to_json_binary(&(balance + local_balance)).map_err(NeutronError::Std)
}

fn query_config(deps: Deps) -> NeutronResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config).map_err(NeutronError::Std)
}

fn query_ica(deps: Deps) -> NeutronResult<Binary> {
    let ica = ICA.load(deps.storage)?;
    to_json_binary(&ica).map_err(NeutronError::Std)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterICA {} => execute_register_ica(deps, info),
        ExecuteMsg::UpdateConfig { new_config } => {
            execute_update_config(deps, env, info, *new_config)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::IBCTransfer {} => execute_ibc_transfer(deps, env, info),
        ExecuteMsg::Stake { items } => execute_stake(deps, env, info, items),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let attrs = vec![
        attr("action", "update_config"),
        attr("new_config", format!("{:?}", new_config)),
    ];
    if let Some(port_id) = new_config.port_id {
        config.port_id = port_id;
    }

    if let Some(connection_id) = new_config.connection_id {
        config.connection_id = connection_id;
    }
    if let Some(ibc_fees) = new_config.ibc_fees {
        config.ibc_fees = ibc_fees;
    }
    if let Some(timeout) = new_config.timeout {
        config.timeout = timeout;
    }
    if let Some(local_denom) = new_config.local_denom {
        config.local_denom = local_denom;
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_register_ica(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("action", "register_ica"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
    ];
    check_funds(&info, &config, config.ibc_fees.register_fee)?;
    let register_fee: Uint128 = config.ibc_fees.register_fee;
    let register_msg = ICA.register(
        deps.storage,
        config.connection_id,
        ICA_ID,
        coin(register_fee.u128(), config.local_denom),
    )?;
    Ok(response("register-ica", CONTRACT_NAME, attrs).add_message(register_msg))
}

fn execute_stake(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    check_funds(
        &info,
        &config,
        config.ibc_fees.ack_fee + config.ibc_fees.recv_fee + config.ibc_fees.timeout_fee,
    )?;
    if !config.allowed_addresses.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    let amount = NON_STAKED_BALANCE.load(deps.storage)?;
    ensure!(
        amount > Uint128::zero(),
        ContractError::InvalidFunds {
            reason: "no funds to stake".to_string()
        }
    );
    let sum = items
        .iter()
        .fold(Uint128::zero(), |acc, (_, amount)| acc + *amount);
    ensure!(
        amount >= sum,
        ContractError::InvalidFunds {
            reason: "not enough funds to stake".to_string()
        }
    );
    let attrs = vec![
        attr("action", "stake"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
        attr("amount", amount.to_string()),
    ];
    let fee = IbcFee {
        recv_fee: uint_into_vec_coin(config.ibc_fees.recv_fee, &config.local_denom),
        ack_fee: uint_into_vec_coin(config.ibc_fees.ack_fee, &config.local_denom),
        timeout_fee: uint_into_vec_coin(config.ibc_fees.timeout_fee, &config.local_denom),
    };
    let ica = ICA.get_address(deps.storage)?;
    let puppeteer_ica = config
        .puppeteer_ica
        .ok_or(ContractError::Std(StdError::generic_err(
            "puppeteer_ica not set",
        )))?;
    let grant_msg = MsgExec {
        grantee: ica.to_string(),
        msgs: items
            .iter()
            .map(|(validator, amount)| cosmos_sdk_proto::Any {
                type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                value: MsgDelegate {
                    delegator_address: puppeteer_ica.to_string(),
                    validator_address: validator.to_string(),
                    amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                        denom: config.remote_denom.to_string(),
                        amount: amount.to_string(),
                    }),
                }
                .encode_to_vec(),
            })
            .collect(),
    };
    let any_msgs: Vec<neutron_sdk::bindings::types::ProtobufAny> =
        vec![prepare_any_msg(grant_msg, "/cosmos.authz.v1beta1.MsgExec")?];
    let cosmos_msg = NeutronMsg::submit_tx(
        config.connection_id,
        ICA_ID.to_string(),
        any_msgs,
        "".to_string(),
        config.timeout,
        fee,
    );
    let submsg: SubMsg<NeutronMsg> = msg_with_sudo_callback(
        deps,
        cosmos_msg,
        Transaction::Stake { amount },
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;
    Ok(response("stake", CONTRACT_NAME, attrs).add_submessage(submsg))
}

fn execute_ibc_transfer(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    check_funds(
        &info,
        &config,
        config.ibc_fees.ack_fee + config.ibc_fees.recv_fee + config.ibc_fees.timeout_fee,
    )?;
    must_pay(&info, &config.base_denom)?;
    let coin = info
        .funds
        .iter()
        .find(|coin| coin.denom == config.base_denom)
        .ok_or(ContractError::PaymentError(
            cw_utils::PaymentError::NoFunds {},
        ))?;
    NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance + coin.amount))?;
    let attrs = vec![
        attr("action", "ibc_transfer"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
        attr("coin", format!("{:?}", coin)),
    ];
    let fee = IbcFee {
        recv_fee: uint_into_vec_coin(config.ibc_fees.recv_fee, &config.local_denom),
        ack_fee: uint_into_vec_coin(config.ibc_fees.ack_fee, &config.local_denom),
        timeout_fee: uint_into_vec_coin(config.ibc_fees.timeout_fee, &config.local_denom),
    };
    let ica = ICA.get_address(deps.storage)?;
    let msg = NeutronMsg::IbcTransfer {
        source_port: config.port_id.to_string(),
        source_channel: config.transfer_channel_id,
        token: coin.clone(),
        sender: env.contract.address.to_string(),
        receiver: ica.to_string(),
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: None,
            revision_height: None,
        },
        timeout_timestamp: env.block.time.plus_seconds(config.timeout).nanos(),
        memo: "".to_string(),
        fee,
    };

    let submsg: SubMsg<NeutronMsg> = msg_with_sudo_callback(
        deps,
        msg,
        Transaction::IBCTransfer {
            amount: coin.amount,
        },
        ReplyMsg::IbcTransfer.to_reply_id(),
    )?;

    Ok(response("ibc_transfer", CONTRACT_NAME, attrs).add_submessage(submsg))
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_timeout(deps, env, request),
        SudoMsg::OpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => sudo_open_ack(
            deps,
            env,
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        ),
        _ => unimplemented!(),
    }
}

pub fn sudo_open_ack(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _port_id: String,
    _channel_id: String,
    _counterparty_channel_id: String,
    counterparty_version: String,
) -> NeutronResult<Response> {
    let parsed_version: Result<OpenAckVersion, _> =
        serde_json_wasm::from_str(counterparty_version.as_str());
    if let Ok(parsed_version) = parsed_version {
        ICA.set_address(deps.storage, parsed_version.address)?;
        Ok(Response::default())
    } else {
        Err(NeutronError::Std(StdError::generic_err(
            "can't parse version",
        )))
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
    let _seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let msg_data: TxMsgData = TxMsgData::decode(data.as_slice())?;
    deps.api
        .debug(&format!("WASMDEBUG: msg_data: data: {msg_data:?}"));
    let tx_state = TX_STATE.load(deps.storage)?;
    if let Some(Transaction::Stake { amount }) = tx_state.transaction {
        NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance - amount))?;
    }
    TX_STATE.save(deps.storage, &TxState::default())?;
    Ok(response("sudo-response", CONTRACT_NAME, attrs))
}

fn msg_with_sudo_callback<C: Into<CosmosMsg<X>> + Serialize, X>(
    deps: DepsMut<NeutronQuery>,
    msg: C,
    transaction: Transaction,
    payload_id: u64,
) -> StdResult<SubMsg<X>> {
    TX_STATE.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::InProgress,
            seq_id: None,
            transaction: Some(transaction),
        },
    )?;
    Ok(SubMsg::reply_on_success(msg, payload_id))
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
    ICA.set_timeout(deps.storage)?;
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_timeout: request: {request:?}",
        request = request
    ));
    TX_STATE.save(deps.storage, &TxState::default())?;
    Ok(response("sudo-timeout", CONTRACT_NAME, attrs))
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
    let _seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let tx_state = TX_STATE.load(deps.storage)?;
    if let Some(Transaction::IBCTransfer { amount }) = tx_state.transaction {
        NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance - amount))?;
    }
    TX_STATE.save(deps.storage, &TxState::default())?;
    Ok(response("sudo-error", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

fn uint_into_vec_coin(amount: Uint128, denom: &String) -> Vec<Coin> {
    vec![Coin::new(amount.u128(), denom)]
}

fn check_funds(info: &MessageInfo, config: &Config, needed_amount: Uint128) -> ContractResult<()> {
    let info_amount = must_pay(info, &config.local_denom)?;
    ensure!(
        info_amount >= needed_amount,
        ContractError::InvalidFunds {
            reason: format!(
                "invalid amount: expected at least {}, got {}",
                needed_amount, info_amount
            )
        }
    );
    Ok(())
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match ReplyMsg::from_reply_id(msg.id) {
        ReplyMsg::SudoPayload => submit_tx_reply(deps, msg),
        ReplyMsg::IbcTransfer => submit_ibc_transfer_reply(deps, msg),
    }
}

pub fn submit_tx_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    let resp: MsgSubmitTxResponse = serde_json_wasm::from_slice(
        msg.result
            .into_result()
            .map_err(StdError::generic_err)?
            .data
            .ok_or_else(|| StdError::generic_err("no result"))?
            .as_slice(),
    )
    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?;
    let seq_id = resp.sequence_id;
    let channel_id = resp.channel;
    let mut tx_state: TxState = TX_STATE.load(deps.storage)?;
    tx_state.seq_id = Some(seq_id);
    tx_state.status = TxStateStatus::WaitingForAck;
    TX_STATE.save(deps.storage, &tx_state)?;
    let atts = vec![
        attr("channel_id", channel_id.to_string()),
        attr("seq_id", seq_id.to_string()),
    ];
    Ok(response(
        "reply-tx-payload-received",
        "puppeteer-base",
        atts,
    ))
}

pub fn submit_ibc_transfer_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
    let resp: MsgIbcTransferResponse = serde_json_wasm::from_slice(
        msg.result
            .into_result()
            .map_err(StdError::generic_err)?
            .data
            .ok_or_else(|| StdError::generic_err("no result"))?
            .as_slice(),
    )
    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?;
    deps.api
        .debug(format!("WASMDEBUG: prepare_sudo_payload received; resp: {resp:?}").as_str());
    let seq_id = resp.sequence_id;
    let channel_id = resp.channel;
    let mut tx_state: TxState = TX_STATE.load(deps.storage)?;
    tx_state.seq_id = Some(seq_id);
    tx_state.status = TxStateStatus::WaitingForAck;
    TX_STATE.save(deps.storage, &tx_state)?;
    let atts = vec![
        attr("channel_id", channel_id.to_string()),
        attr("seq_id", seq_id.to_string()),
    ];
    Ok(response(
        "reply-ibc-transfer-payload-received",
        "puppeteer-base",
        atts,
    ))
}
