use crate::error::{ContractError, ContractResult};
use cosmos_sdk_proto::cosmos::authz::v1beta1::MsgExec;
use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate;
use cosmos_sdk_proto::traits::MessageExt;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    attr, ensure, to_json_binary, CosmosMsg, Deps, Reply, StdError, SubMsg, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use drop_helpers::{
    answer::response, ibc_client_state::query_client_state, ibc_fee::query_ibc_fee,
    interchain::prepare_any_msg, validation::validate_addresses,
};
use drop_proto::proto::initia::mstaking::v1::InitiaMsgDelegate;
use drop_staking_base::state::staker::{ChainType, CHAIN_TYPE};
use drop_staking_base::{
    msg::staker::{
        ExecuteMsg, InstantiateMsg, MigrateMsg, OpenAckVersion, QueryMsg, ReceiverExecuteMsg,
        ResponseHookErrorMsg, ResponseHookMsg, ResponseHookSuccessMsg,
    },
    state::staker::{
        Config, ConfigOptional, ReplyMsg, Transaction, TxState, TxStateStatus, CONFIG, ICA, ICA_ID,
        NON_STAKED_BALANCE, TX_STATE,
    },
};
use neutron_sdk::{
    bindings::{
        msg::{MsgIbcTransferResponse, MsgSubmitTxResponse, NeutronMsg},
        query::NeutronQuery,
    },
    sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, SudoMsg},
    NeutronError, NeutronResult,
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LOCAL_DENOM: &str = "untrn";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
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
    let allowed_senders = validate_addresses(
        deps.as_ref().into_empty(),
        msg.allowed_senders.as_ref(),
        None,
    )?;
    CONFIG.save(
        deps.storage,
        &Config {
            port_id: msg.port_id,
            transfer_channel_id: msg.transfer_channel_id,
            connection_id: msg.connection_id,
            timeout: msg.timeout,
            remote_denom: msg.remote_denom,
            base_denom: msg.base_denom,
            allowed_senders,
            puppeteer_ica: None,
            min_ibc_transfer: msg.min_ibc_transfer,
            min_staking_amount: msg.min_staking_amount,
        },
    )?;
    NON_STAKED_BALANCE.save(deps.storage, &Uint128::zero())?;
    CHAIN_TYPE.save(
        deps.storage,
        &msg.chain_type.unwrap_or(ChainType::default()),
    )?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Ica {} => query_ica(deps),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::AllBalance {} => query_all_balance(deps, env),
        QueryMsg::TxState {} => query_tx_state(deps, env),
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            to_json_binary(&ownership).map_err(NeutronError::Std)
        }
    }
}

fn query_tx_state(deps: Deps, _env: Env) -> NeutronResult<Binary> {
    let tx_state = TX_STATE.load(deps.storage)?;
    to_json_binary(&tx_state).map_err(NeutronError::Std)
}

fn query_non_staked_balance(deps: Deps, _env: Env) -> NeutronResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    Ok(to_json_binary(&(balance))?)
}

fn query_all_balance(deps: Deps, env: Env) -> NeutronResult<Binary> {
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

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterICA {} => execute_register_ica(deps, info),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, *new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::IBCTransfer {} => execute_ibc_transfer(deps, env),
        ExecuteMsg::Stake { items } => execute_stake(deps, info, items),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let attrs = vec![
        attr("action", "update_config"),
        attr("new_config", format!("{:?}", new_config)),
    ];
    if let Some(timeout) = new_config.timeout {
        config.timeout = timeout;
    }
    if let Some(allowed_senders) = new_config.allowed_senders {
        let allowed_senders =
            validate_addresses(deps.as_ref().into_empty(), allowed_senders.as_ref(), None)?;
        config.allowed_senders = allowed_senders;
    }
    if let Some(puppeteer_ica) = new_config.puppeteer_ica {
        config.puppeteer_ica = Some(puppeteer_ica);
    }
    if let Some(min_ibc_transfer) = new_config.min_ibc_transfer {
        config.min_ibc_transfer = min_ibc_transfer;
    }
    if let Some(min_staking_amount) = new_config.min_staking_amount {
        config.min_staking_amount = min_staking_amount;
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
    let register_fee = info
        .funds
        .into_iter()
        .find(|f| f.denom == LOCAL_DENOM)
        .ok_or(ContractError::InvalidFunds {
            reason: format!("missing fee in denom {}", LOCAL_DENOM),
        })?;
    let register_msg = ICA.register(deps.storage, config.connection_id, ICA_ID, register_fee)?;
    Ok(response("register-ica", CONTRACT_NAME, attrs).add_message(register_msg))
}

fn execute_stake(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    if !config.allowed_senders.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::Idle,
        ContractError::InvalidState {
            reason: "tx_state is not idle".to_string()
        }
    );
    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;
    ensure!(
        non_staked_balance > Uint128::zero(),
        ContractError::InvalidFunds {
            reason: "no funds to stake".to_string()
        }
    );
    let amount_to_stake = items
        .iter()
        .fold(Uint128::zero(), |acc, (_, amount)| acc + *amount);
    ensure!(
        amount_to_stake >= config.min_staking_amount,
        ContractError::InvalidFunds {
            reason: "amount is less than min_staking_amount".to_string()
        }
    );
    ensure!(
        non_staked_balance >= amount_to_stake,
        ContractError::InvalidFunds {
            reason: "not enough funds to stake".to_string()
        }
    );
    let attrs = vec![
        attr("action", "stake"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
        attr("amount_to_stake", amount_to_stake.to_string()),
    ];
    let fee = query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?;
    let ica = ICA.get_address(deps.storage)?;
    let puppeteer_ica = config
        .puppeteer_ica
        .ok_or(ContractError::Std(StdError::generic_err(
            "puppeteer_ica not set",
        )))?;
    let mut delegations = vec![];
    let chain_type = CHAIN_TYPE.load(deps.storage).unwrap_or_default();
    for (validator, amount) in items {
        delegations.push(get_delegate_msg(
            chain_type.clone(),
            puppeteer_ica.to_string(),
            validator,
            config.remote_denom.to_string(),
            amount.to_string(),
        )?);
    }
    let grant_msg = MsgExec {
        grantee: ica.to_string(),
        msgs: delegations,
    };
    let bank_send_msg = MsgSend {
        from_address: ica.to_string(),
        to_address: puppeteer_ica,
        amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: config.remote_denom.to_string(),
            amount: amount_to_stake.to_string(),
        }],
    };
    let any_msgs: Vec<neutron_sdk::bindings::types::ProtobufAny> = vec![
        prepare_any_msg(bank_send_msg, "/cosmos.bank.v1beta1.MsgSend")?,
        prepare_any_msg(grant_msg, "/cosmos.authz.v1beta1.MsgExec")?,
    ];
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
        Transaction::Stake {
            amount: non_staked_balance,
        },
        ReplyMsg::SudoPayload.to_reply_id(),
        Some(info.sender.to_string()),
    )?;
    Ok(response("stake", CONTRACT_NAME, attrs).add_submessage(submsg))
}

fn execute_ibc_transfer(
    deps: DepsMut<NeutronQuery>,
    env: Env,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::Idle,
        ContractError::InvalidState {
            reason: "tx_state is not idle".to_string()
        }
    );
    let pending_coin = deps
        .querier
        .query_balance(&env.contract.address, config.base_denom)?;
    ensure!(
        pending_coin.amount >= config.min_ibc_transfer,
        ContractError::InvalidFunds {
            reason: "amount is less than min_ibc_transfer".to_string()
        }
    );
    NON_STAKED_BALANCE.update(deps.storage, |balance| {
        StdResult::Ok(balance + pending_coin.amount)
    })?;
    let attrs = vec![
        attr("action", "ibc_transfer"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
        attr("pending_amount", pending_coin.amount),
    ];
    let fee = query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?;
    let ica = ICA.get_address(deps.storage)?;
    let msg = NeutronMsg::IbcTransfer {
        source_port: config.port_id.to_string(),
        source_channel: config.transfer_channel_id,
        token: pending_coin.clone(),
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
            amount: pending_coin.amount,
        },
        ReplyMsg::IbcTransfer.to_reply_id(),
        None,
    )?;

    Ok(response("ibc_transfer", CONTRACT_NAME, attrs).add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> ContractResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: STAKER sudo: {:?}", msg).as_str());
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
    port_id: String,
    channel_id: String,
    _counterparty_channel_id: String,
    counterparty_version: String,
) -> ContractResult<Response> {
    let parsed_version: Result<OpenAckVersion, _> =
        serde_json_wasm::from_str(counterparty_version.as_str());
    if let Ok(parsed_version) = parsed_version {
        ICA.set_address(deps.storage, parsed_version.address, port_id, channel_id)?;
        Ok(Response::default())
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "can't parse version",
        )))
    }
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    request: RequestPacket,
    _data: Binary,
) -> ContractResult<Response> {
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let channel_id = request
        .clone()
        .source_channel
        .ok_or_else(|| StdError::generic_err("source_channel not found"))?;
    let port_id = request
        .clone()
        .source_port
        .ok_or_else(|| StdError::generic_err("source_port not found"))?;
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.seq_id == Some(seq_id),
        ContractError::InvalidState {
            reason: "seq_id does not match".to_string()
        }
    );
    ensure!(
        tx_state.status == TxStateStatus::WaitingForAck,
        ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    );
    let reply_to = tx_state.reply_to;
    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    if let Transaction::Stake { amount } = transaction {
        NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance - amount))?;
    }
    TX_STATE.save(deps.storage, &TxState::default())?;

    let client_state = query_client_state(&deps.as_ref(), channel_id, port_id)?;
    let remote_height = client_state
        .identified_client_state
        .ok_or_else(|| StdError::generic_err("IBC client state identified_client_state not found"))?
        .client_state
        .latest_height
        .ok_or_else(|| StdError::generic_err("IBC client state latest_height not found"))?
        .revision_height;

    let mut msgs = vec![];
    if let Some(reply_to) = reply_to {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to,
            msg: to_json_binary(&ReceiverExecuteMsg::StakerHook(ResponseHookMsg::Success(
                ResponseHookSuccessMsg {
                    request_id: seq_id,
                    request: request.clone(),
                    transaction: transaction.clone(),
                    local_height: env.block.height,
                    remote_height: remote_height.u64(),
                },
            )))?,
            funds: vec![],
        }));
    }
    Ok(response("sudo-response", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn msg_with_sudo_callback<C: Into<CosmosMsg<X>> + Serialize, X>(
    deps: DepsMut<NeutronQuery>,
    msg: C,
    transaction: Transaction,
    payload_id: u64,
    reply_to: Option<String>,
) -> StdResult<SubMsg<X>> {
    TX_STATE.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::InProgress,
            seq_id: None,
            transaction: Some(transaction),
            reply_to,
        },
    )?;
    Ok(SubMsg::reply_on_success(msg, payload_id))
}

fn sudo_timeout(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
) -> ContractResult<Response> {
    let attrs = vec![
        attr("action", "sudo_timeout"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    ICA.set_timeout(deps.storage)?;
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let tx_state = TX_STATE.load(deps.storage)?;
    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    if let Transaction::IBCTransfer { amount } = transaction {
        NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance - amount))?;
    }
    let mut msgs = vec![];
    if let Some(reply_to) = tx_state.reply_to {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to,
            msg: to_json_binary(&ReceiverExecuteMsg::StakerHook(ResponseHookMsg::Error(
                ResponseHookErrorMsg {
                    request_id: seq_id,
                    request,
                    transaction,
                    details: "timeout".to_string(),
                },
            )))?,
            funds: vec![],
        }));
    }
    TX_STATE.save(deps.storage, &TxState::default())?;
    Ok(response("sudo-timeout", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    details: String,
) -> ContractResult<Response> {
    let attrs = vec![
        attr("action", "sudo_error"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
        attr("details", details.clone()),
    ];
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::WaitingForAck,
        ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    );
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    if let Transaction::IBCTransfer { amount } = transaction {
        NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance - amount))?;
    }
    let mut msgs = vec![];
    if let Some(reply_to) = tx_state.reply_to {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to,
            msg: to_json_binary(&ReceiverExecuteMsg::StakerHook(ResponseHookMsg::Error(
                ResponseHookErrorMsg {
                    request_id: seq_id,
                    request,
                    transaction,
                    details,
                },
            )))?,
            funds: vec![],
        }));
    }
    TX_STATE.save(deps.storage, &TxState::default())?;
    Ok(response("sudo-error", CONTRACT_NAME, attrs).add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match ReplyMsg::from_reply_id(msg.id) {
        ReplyMsg::SudoPayload => submit_tx_reply(deps, msg),
        ReplyMsg::IbcTransfer => submit_ibc_transfer_reply(deps, msg),
    }
}

fn submit_tx_reply(deps: DepsMut, msg: Reply) -> StdResult<Response> {
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

fn get_delegate_msg(
    chain_type: ChainType,
    delegator: String,
    validator: String,
    denom: String,
    amount: String,
) -> ContractResult<cosmos_sdk_proto::Any> {
    match chain_type {
        ChainType::BasicCosmos => Ok(cosmos_sdk_proto::Any {
            type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
            value: MsgDelegate {
                delegator_address: delegator,
                validator_address: validator,
                amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin { denom, amount }),
            }
            .to_bytes()?,
        }),
        ChainType::Initia => Ok(cosmos_sdk_proto::Any {
            type_url: "/initia.mstaking.v1.MsgDelegate".to_string(),
            value: InitiaMsgDelegate {
                delegator_address: delegator,
                validator_address: validator,
                amount: vec![drop_proto::proto::cosmos::base::v1beta1::Coin { denom, amount }],
            }
            .to_bytes()?,
        }),
    }
}
