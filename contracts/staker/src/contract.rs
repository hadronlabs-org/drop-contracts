use crate::error::{ContractError, ContractResult};
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, CosmosMsg, Deps, Reply, StdError, SubMsg, Uint128,
    WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use drop_helpers::{answer::response, ibc_client_state::query_client_state};
use drop_puppeteer_base::msg::IBCTransferReason;
use drop_staking_base::state::staker::PUPPETEER_TRANSFER_REPLY_ID;
use drop_staking_base::{
    msg::staker::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::staker::{Config, ConfigOptional, CONFIG, NON_STAKED_BALANCE},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    sudo::msg::{RequestPacket, SudoMsg},
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
    let puppeteer_address = deps.api.addr_validate(msg.puppeteer_address.as_ref())?;
    CONFIG.save(
        deps.storage,
        &Config {
            remote_denom: msg.remote_denom,
            base_denom: msg.base_denom,
            puppeteer_address: puppeteer_address.into_string(),
            min_ibc_transfer: msg.min_ibc_transfer,
        },
    )?;
    NON_STAKED_BALANCE.save(deps.storage, &Uint128::zero())?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::AllBalance {} => query_all_balance(deps, env),
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            to_json_binary(&ownership).map_err(NeutronError::Std)
        }
    }
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

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, *new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::PuppeteerTransfer {} => execute_puppeteer_transfer(deps, env),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
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
    if let Some(puppeteer_address) = new_config.puppeteer_address {
        config.puppeteer_address = puppeteer_address;
    }
    if let Some(min_ibc_transfer) = new_config.min_ibc_transfer {
        config.min_ibc_transfer = min_ibc_transfer;
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_puppeteer_transfer(
    deps: DepsMut<NeutronQuery>,
    env: Env,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

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
        attr("action", "puppeteer_transfer"),
        attr("pending_amount", pending_coin.amount),
    ];

    let puppeteer_transfer = SubMsg::reply_on_error(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.puppeteer_address.to_string(),
            msg: to_json_binary(
                &drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
                    reason: IBCTransferReason::Stake,
                    reply_to: env.contract.address.to_string(),
                },
            )?,
            funds: vec![pending_coin],
        }),
        PUPPETEER_TRANSFER_REPLY_ID,
    );

    Ok(response("puppeteer_transfer", CONTRACT_NAME, attrs).add_submessage(puppeteer_transfer))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_address,
        ContractError::Unauthorized {}
    );
    match msg.clone() {
        drop_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) => {
            match &success_msg.transaction {
                drop_puppeteer_base::msg::Transaction::Stake { items, .. } => {
                    let mut sum = 0u128;
                    for item in items {
                        let (_remote_denom, _shares_amount, real_amount) = LSM_SHARES_TO_REDEEM
                            .load(deps.storage, item.local_denom.to_string())?;
                        sum += real_amount.u128();
                        LSM_SHARES_TO_REDEEM.remove(deps.storage, item.local_denom.to_string());
                    }
                    TOTAL_LSM_SHARES.update(deps.storage, |one| StdResult::Ok(one - sum))?;
                    LAST_LSM_REDEEM.save(deps.storage, &env.block.time.seconds())?;
                }
                _ => {}
            }
        }
        drop_puppeteer_base::msg::ResponseHookMsg::Error(error_msg) => {
            if let drop_puppeteer_base::msg::Transaction::IBCTransfer {
                denom,
                amount,
                reason,
                recipient: _,
            } = &error_msg.transaction
            {
                if *reason == IBCTransferReason::LSMShare {
                    let current_pending =
                        PENDING_LSM_SHARES.may_load(deps.storage, denom.to_string())?;
                    if let Some((remote_denom, shares_amount, real_amount)) = current_pending {
                        let sent_amount = Uint128::from(*amount);
                        LSM_SHARES_TO_REDEEM.update(deps.storage, denom.to_string(), |one| {
                            let mut new =
                                one.unwrap_or((remote_denom, Uint128::zero(), Uint128::zero()));
                            new.1 += sent_amount;
                            new.2 += real_amount;
                            StdResult::Ok(new)
                        })?;
                        if shares_amount == sent_amount {
                            PENDING_LSM_SHARES.remove(deps.storage, denom.to_string());
                        } else {
                            PENDING_LSM_SHARES.update(deps.storage, denom.to_string(), |one| {
                                match one {
                                    Some(one) => {
                                        let mut new = one;
                                        new.1 -= Uint128::from(*amount);
                                        new.2 -= real_amount;
                                        StdResult::Ok(new)
                                    }
                                    None => unreachable!("denom should be in the map"),
                                }
                            })?;
                        }
                    }
                }
            }
        }
    }

    LAST_PUPPETEER_RESPONSE.save(deps.storage, &msg)?;

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> ContractResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: STAKER sudo: {:?}", msg).as_str());
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_timeout(deps, env, request),
        _ => unimplemented!(),
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
    match msg.id {
        PUPPETEER_TRANSFER_REPLY_ID => puppeteer_transfer_reply(deps, msg),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn puppeteer_transfer_reply(_deps: Deps, msg: Reply) -> ContractResult<Response> {
    if let SubMsgResult::Err(err) = msg.result {
        return Err(ContractError::PuppeteerError { message: err });
    }

    Ok(Response::new())
}
