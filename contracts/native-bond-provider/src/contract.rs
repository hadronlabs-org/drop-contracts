use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Attribute, Coin, CosmosMsg, Decimal, Deps, Empty,
    Reply, StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_helpers::ibc_fee::query_ibc_fee;
use drop_puppeteer_base::msg::ReceiverExecuteMsg;
use drop_staking_base::error::native_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::core::LastPuppeteerResponse;
use drop_staking_base::msg::native_bond_provider::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::native_bond_provider::{
    Config, ConfigOptional, ReplyMsg, Transaction, TxState, TxStateStatus, CONFIG,
    LAST_PUPPETEER_RESPONSE, NON_STAKED_BALANCE, TX_STATE,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, SudoMsg};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const LOCAL_DENOM: &str = "untrn";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let puppeteer_contract = deps.api.addr_validate(&msg.puppeteer_contract)?;
    let core_contract = deps.api.addr_validate(&msg.core_contract)?;
    let strategy_contract = deps.api.addr_validate(&msg.strategy_contract)?;

    let config = &Config {
        puppeteer_contract: puppeteer_contract.clone(),
        core_contract: core_contract.clone(),
        strategy_contract: strategy_contract.clone(),
        base_denom: msg.base_denom.to_string(),
        min_ibc_transfer: msg.min_ibc_transfer,
        min_stake_amount: msg.min_stake_amount,
        transfer_channel_id: msg.transfer_channel_id.clone(),
        port_id: msg.port_id.clone(),
        timeout: msg.timeout,
    };
    CONFIG.save(deps.storage, config)?;

    NON_STAKED_BALANCE.save(deps.storage, &Uint128::zero())?;
    TX_STATE.save(deps.storage, &TxState::default())?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("puppeteer_contract", puppeteer_contract.into_string()),
            attr("core_contract", core_contract.into_string()),
            attr("strategy_contract", strategy_contract.into_string()),
            attr("min_ibc_transfer", msg.min_ibc_transfer),
            attr("min_stake_amount", msg.min_stake_amount),
            attr("base_denom", msg.base_denom),
            attr("port_id", msg.port_id),
            attr("transfer_channel_id", msg.transfer_channel_id),
            attr("timeout", msg.timeout.to_string()),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::CanBond { denom } => query_can_bond(deps, denom),
        QueryMsg::CanProcessOnIdle {} => {
            let config = CONFIG.load(deps.storage)?;

            let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;

            Ok(to_json_binary(&query_can_process_on_idle(
                deps,
                &env,
                &config,
                non_staked_balance,
            )?)?)
        }
        QueryMsg::TokensAmount {
            coin,
            exchange_rate,
        } => query_token_amount(deps, coin, exchange_rate),
        QueryMsg::AsyncTokensAmount {} => Ok(to_json_binary(&Uint128::zero())?),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::AllBalance {} => query_all_balance(deps, env),
        QueryMsg::TxState {} => query_tx_state(deps, env),
        QueryMsg::LastPuppeteerResponse {} => Ok(to_json_binary(&LastPuppeteerResponse {
            response: LAST_PUPPETEER_RESPONSE.may_load(deps.storage)?,
        })?),
    }
}

fn query_tx_state(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let tx_state = TX_STATE.load(deps.storage)?;
    Ok(to_json_binary(&tx_state)?)
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_non_staked_balance(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    Ok(to_json_binary(&(balance))?)
}

fn query_all_balance(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let local_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), config.base_denom)?
        .amount;
    to_json_binary(&(balance + local_balance)).map_err(ContractError::Std)
}

fn query_can_bond(deps: Deps<NeutronQuery>, denom: String) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    Ok(to_json_binary(&can_bond(config.base_denom, denom))?)
}

fn query_can_process_on_idle(
    deps: Deps<NeutronQuery>,
    _env: &Env,
    config: &Config,
    non_staked_balance: Uint128,
) -> ContractResult<bool> {
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::Idle,
        ContractError::InvalidState {
            reason: "tx_state is not idle".to_string()
        }
    );

    if non_staked_balance < config.min_stake_amount {
        return Err(ContractError::NotEnoughToDelegate {
            min_stake_amount: config.min_stake_amount,
            non_staked_balance,
        });
    }

    Ok(true)
}

fn query_token_amount(
    deps: Deps<NeutronQuery>,
    coin: Coin,
    exchange_rate: Decimal,
) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    if can_bond(config.base_denom, coin.denom) {
        let issue_amount = coin.amount * (Decimal::one() / exchange_rate);

        return Ok(to_json_binary(&issue_amount)?);
    }

    Err(ContractError::InvalidDenom {})
}

fn can_bond(base_denom: String, denom: String) -> bool {
    base_denom == denom
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::Bond {} => execute_bond(deps, info),
        ExecuteMsg::ProcessOnIdle {} => execute_process_on_idle(deps, env, info),
        ExecuteMsg::IBCTransfer {} => execute_ibc_transfer(deps, env),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut state = CONFIG.load(deps.storage)?;
    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(puppeteer_contract) = new_config.puppeteer_contract {
        state.puppeteer_contract = deps.api.addr_validate(puppeteer_contract.as_ref())?;
        attrs.push(attr("puppeteer_contract", puppeteer_contract))
    }

    if let Some(core_contract) = new_config.core_contract {
        state.core_contract = deps.api.addr_validate(core_contract.as_ref())?;
        attrs.push(attr("core_contract", core_contract))
    }

    if let Some(strategy_contract) = new_config.strategy_contract {
        state.strategy_contract = deps.api.addr_validate(strategy_contract.as_ref())?;
        attrs.push(attr("strategy_contract", strategy_contract))
    }

    if let Some(base_denom) = new_config.base_denom {
        state.base_denom = base_denom.to_string();
        attrs.push(attr("base_denom", base_denom));
    }

    if let Some(min_ibc_transfer) = new_config.min_ibc_transfer {
        state.min_ibc_transfer = min_ibc_transfer;
        attrs.push(attr("min_ibc_transfer", min_ibc_transfer));
    }

    if let Some(min_stake_amount) = new_config.min_stake_amount {
        state.min_stake_amount = min_stake_amount;
        attrs.push(attr("min_stake_amount", min_stake_amount));
    }

    if let Some(port_id) = new_config.port_id {
        state.port_id = port_id.clone();
        attrs.push(attr("port_id", port_id));
    }

    if let Some(transfer_channel_id) = new_config.transfer_channel_id {
        state.transfer_channel_id = transfer_channel_id.clone();
        attrs.push(attr("transfer_channel_id", transfer_channel_id));
    }

    if let Some(timeout) = new_config.timeout {
        state.timeout = timeout;
        attrs.push(attr("timeout", timeout.to_string()));
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let Coin { amount, denom } = cw_utils::one_coin(&info)?;
    let config = CONFIG.load(deps.storage)?;

    if denom != config.base_denom {
        return Err(ContractError::InvalidDenom {});
    }

    Ok(response(
        "bond",
        CONTRACT_NAME,
        [attr_coin("received_funds", amount.to_string(), denom)],
    ))
}

fn execute_process_on_idle(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;
    query_can_process_on_idle(deps.as_ref(), &env, &config, non_staked_balance)?;

    let to_delegate: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        &config.strategy_contract,
        &drop_staking_base::msg::strategy::QueryMsg::CalcDeposit {
            deposit: non_staked_balance,
        },
    )?;

    let attrs = vec![attr("action", "process_on_idle")];

    let puppeteer_delegation_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract.to_string(),
        msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
            items: to_delegate,
            reply_to: env.contract.address.to_string(),
        })?,
        funds: vec![],
    });

    let submsg: SubMsg<NeutronMsg> = msg_with_reply_callback(
        deps,
        puppeteer_delegation_msg,
        Transaction::Stake {
            amount: non_staked_balance,
        },
        ReplyMsg::Bond.to_reply_id(),
    )?;

    Ok(
        response("process_on_idle", CONTRACT_NAME, Vec::<Attribute>::new())
            .add_submessage(submsg)
            .add_attributes(attrs),
    )
}

// pub fn get_stake_bond_msg<T>(
//     deps: Deps<NeutronQuery>,
//     _env: &Env,
//     config: &Config,
//     info: &MessageInfo,
// ) -> ContractResult<Option<CosmosMsg<T>>> {
//     let staker_pending_stake: Result<Uint128, _> = deps.querier.query_wasm_smart(
//         config.staker_contract.to_string(),
//         &drop_staking_base::msg::staker::QueryMsg::NonStakedBalance {},
//     );
//     if let Ok(staker_pending_stake) = staker_pending_stake {
//         if staker_pending_stake.is_zero() {
//             return Ok(None);
//         }
//         let to_delegate: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
//             &config.strategy_contract,
//             &drop_staking_base::msg::strategy::QueryMsg::CalcDeposit {
//                 deposit: staker_pending_stake,
//             },
//         )?;
//         return Ok(Some(CosmosMsg::<T>::Wasm(WasmMsg::Execute {
//             contract_addr: config.staker_contract.to_string(),
//             msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
//                 items: to_delegate,
//                 reply_to: config.staker_contract.to_string(),
//             })?,
//             funds: info.funds.clone(),
//         })));
//     }
//     Ok(None)
// }

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

    let puppeteer_ica: drop_helpers::ica::IcaState = deps.querier.query_wasm_smart(
        &config.puppeteer_contract,
        &drop_puppeteer_base::msg::QueryMsg::<Empty>::Ica {},
    )?;

    if let drop_helpers::ica::IcaState::Registered { ica_address, .. } = puppeteer_ica {
        let attrs = vec![
            attr("action", "puppeteer_transfer"),
            attr("pending_amount", pending_coin.amount),
        ];

        let msg = NeutronMsg::IbcTransfer {
            source_port: config.port_id.clone(),
            source_channel: config.transfer_channel_id.clone(),
            token: pending_coin.clone(),
            sender: env.contract.address.to_string(),
            receiver: ica_address.to_string(),
            timeout_height: RequestPacketTimeoutHeight {
                revision_number: None,
                revision_height: None,
            },
            timeout_timestamp: env.block.time.plus_seconds(config.timeout).nanos(),
            memo: "".to_string(),
            fee: query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?,
        };

        let submsg: SubMsg<NeutronMsg> = msg_with_reply_callback(
            deps,
            msg,
            Transaction::IBCTransfer {
                amount: pending_coin.amount,
            },
            ReplyMsg::IbcTransfer.to_reply_id(),
        )?;

        return Ok(response("puppeteer_transfer", CONTRACT_NAME, attrs).add_submessage(submsg));
    }

    Err(ContractError::IcaNotRegistered {})

    // NON_STAKED_BALANCE.update(deps.storage, |balance| {
    //     StdResult::Ok(balance + pending_coin.amount)
    // })?;

    // let puppeteer_transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: config.puppeteer_contract.to_string(),
    //     msg: to_json_binary(
    //         &drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
    //             reason: IBCTransferReason::Delegate,
    //             reply_to: env.contract.address.to_string(),
    //         },
    //     )?,
    //     funds: vec![pending_coin.clone()],
    // });
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );

    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::WaitingForAck,
        ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    );

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;

    match msg.clone() {
        drop_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) => {
            if let drop_puppeteer_base::msg::Transaction::Stake { items } = success_msg.transaction
            {
                if let Transaction::Stake { .. } = transaction {
                    let amount_to_stake: Uint128 = items.iter().map(|(_, amount)| *amount).sum();

                    NON_STAKED_BALANCE.update(deps.storage, |balance| {
                        StdResult::Ok(balance - amount_to_stake)
                    })?;

                    TX_STATE.save(deps.storage, &TxState::default())?;
                }
            }
        }
        drop_puppeteer_base::msg::ResponseHookMsg::Error(error_msg) => {
            if let drop_puppeteer_base::msg::Transaction::Stake { .. } = error_msg.transaction {
                TX_STATE.save(deps.storage, &TxState::default())?;
            }
        }
    }

    LAST_PUPPETEER_RESPONSE.save(deps.storage, &msg)?;

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PuppeteerHook(msg))?,
        funds: vec![],
    });

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    )
    .add_message(hook_message))
}

fn msg_with_reply_callback<C: Into<CosmosMsg<X>> + Serialize, X>(
    deps: DepsMut<NeutronQuery>,
    msg: C,
    transaction: Transaction,
    payload_id: u64,
) -> StdResult<SubMsg<X>> {
    TX_STATE.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::InProgress,
            transaction: Some(transaction),
        },
    )?;
    Ok(SubMsg::reply_always(msg, payload_id))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    if let SubMsgResult::Err(err) = msg.result {
        return Err(ContractError::PuppeteerError { message: err });
    }

    match ReplyMsg::from_reply_id(msg.id) {
        ReplyMsg::IbcTransfer | ReplyMsg::Bond => puppeteer_reply(deps),
    }
}

fn puppeteer_reply(deps: DepsMut) -> ContractResult<Response> {
    let mut tx_state: TxState = TX_STATE.load(deps.storage)?;
    tx_state.status = TxStateStatus::WaitingForAck;
    TX_STATE.save(deps.storage, &tx_state)?;

    if let Some(Transaction::IBCTransfer { amount }) = tx_state.transaction {
        NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance + amount))?;
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: SudoMsg,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?} block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, .. } => sudo_error(deps, env, request),
        SudoMsg::Timeout { request } => sudo_error(deps, env, request),
        _ => Err(ContractError::MessageIsNotSupported {}),
    }
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_error: request: {request:?}",
        request = request
    ));
    let attrs = vec![
        attr("action", "sudo_error"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];

    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::WaitingForAck,
        ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    );

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;

    if let Transaction::IBCTransfer { amount, .. } = transaction.clone() {
        NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance - amount))?;
    }

    TX_STATE.save(deps.storage, &TxState::default())?;

    deps.api.debug(&format!(
        "WASMDEBUG: sudo_timeout: request: {request:?}",
        request = request
    ));

    Ok(response("sudo-timeout", "puppeteer", attrs))
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    _data: Binary,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug("WASMDEBUG: sudo response");
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::WaitingForAck,
        ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    );

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;

    deps.api.debug(&format!(
        "WASMDEBUG: transaction: {transaction:?}",
        transaction = transaction
    ));
    TX_STATE.save(deps.storage, &TxState::default())?;

    Ok(response("sudo-response", "puppeteer", attrs))
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
