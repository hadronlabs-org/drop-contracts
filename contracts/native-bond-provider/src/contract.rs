use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Attribute, Coin, CosmosMsg, Decimal, Deps, Empty,
    Reply, StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_helpers::get_contracts;
use drop_helpers::ibc_client_state::query_client_state;
use drop_helpers::ibc_fee::query_ibc_fee;
use drop_puppeteer_base::peripheral_hook::{
    IBCTransferReason, ReceiverExecuteMsg, ResponseHookErrorMsg, ResponseHookMsg,
    ResponseHookSuccessMsg, Transaction,
};
use drop_staking_base::error::native_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::core::LastPuppeteerResponse;
use drop_staking_base::msg::native_bond_provider::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::native_bond_provider::{
    Config, ConfigOptional, ReplyMsg, TxState, TxStateStatus, CONFIG, LAST_PUPPETEER_RESPONSE,
    NON_STAKED_BALANCE, TX_STATE,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, SudoMsg};

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
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

    let factory_contract = deps.api.addr_validate(&msg.factory_contract)?;

    let config = &Config {
        factory_contract: factory_contract.clone(),
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
            attr("factory_contract", factory_contract.into_string()),
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
            Ok(to_json_binary(&query_can_process_on_idle(
                deps, &env, &config,
            )?)?)
        }
        QueryMsg::TokensAmount {
            coin,
            exchange_rate,
        } => query_token_amount(deps, coin, exchange_rate),
        QueryMsg::AsyncTokensAmount {} => query_async_tokens_amount(deps, env),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::TxState {} => query_tx_state(deps, env),
        QueryMsg::LastPuppeteerResponse {} => Ok(to_json_binary(&LastPuppeteerResponse {
            response: LAST_PUPPETEER_RESPONSE.may_load(deps.storage)?,
        })?),
        QueryMsg::CanBeRemoved {} => query_can_be_removed(deps, env),
    }
}

fn query_can_be_removed(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let all_balances = deps.querier.query_all_balances(env.contract.address)?;
    let all_balances_except_untrn = all_balances
        .into_iter()
        .filter(|coin| coin.denom != *LOCAL_DENOM.to_string())
        .collect::<Vec<Coin>>();
    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let result = all_balances_except_untrn.is_empty()
        && (non_staked_balance.is_zero())
        && TX_STATE.load(deps.storage)?.status == TxStateStatus::Idle;
    Ok(to_json_binary(&result)?)
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

fn query_async_tokens_amount(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
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
    env: &Env,
    config: &Config,
) -> ContractResult<bool> {
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::Idle,
        ContractError::InvalidState {
            reason: "tx_state is not idle".to_string()
        }
    );

    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let pending_coin = deps
        .querier
        .query_balance(&env.contract.address, config.base_denom.to_string())?;

    ensure!(
        pending_coin.amount >= config.min_ibc_transfer
            || non_staked_balance >= config.min_stake_amount,
        ContractError::NotEnoughToProcessIdle {
            min_stake_amount: config.min_stake_amount,
            non_staked_balance,
            min_ibc_transfer: config.min_ibc_transfer,
            pending_coins: pending_coin.amount,
        }
    );

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
        ExecuteMsg::PeripheralHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
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

    if let Some(factory_contract) = new_config.factory_contract {
        state.factory_contract = deps.api.addr_validate(factory_contract.as_ref())?;
        attrs.push(attr("factory_contract", factory_contract))
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
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let addrs = get_contracts!(deps, config.factory_contract, core_contract);

    ensure_eq!(
        info.sender,
        addrs.core_contract,
        ContractError::Unauthorized {}
    );

    query_can_process_on_idle(deps.as_ref(), &env, &config)?;

    let attrs = vec![attr("action", "process_on_idle")];
    let mut submessages: Vec<SubMsg<NeutronMsg>> = vec![];

    if let Some(lsm_msg) = get_delegation_msg(deps.branch(), &env, &config)? {
        submessages.push(lsm_msg);
    } else if let Some(lsm_msg) = get_ibc_transfer_msg(deps.branch(), &env, &config)? {
        submessages.push(lsm_msg);
    }

    Ok(
        response("process_on_idle", CONTRACT_NAME, Vec::<Attribute>::new())
            .add_submessages(submessages)
            .add_attributes(attrs),
    )
}

fn get_delegation_msg(
    deps: DepsMut<NeutronQuery>,
    env: &Env,
    config: &Config,
) -> ContractResult<Option<SubMsg<NeutronMsg>>> {
    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let addrs = get_contracts!(
        deps,
        config.factory_contract,
        strategy_contract,
        puppeteer_contract
    );

    if non_staked_balance < config.min_stake_amount {
        return Ok(None);
    }

    let to_delegate: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        &addrs.strategy_contract,
        &drop_staking_base::msg::strategy::QueryMsg::CalcDeposit {
            deposit: non_staked_balance,
        },
    )?;
    let puppeteer_delegation_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.puppeteer_contract.to_string(),
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

    Ok(Some(submsg))
}

fn get_ibc_transfer_msg(
    deps: DepsMut<NeutronQuery>,
    env: &Env,
    config: &Config,
) -> ContractResult<Option<SubMsg<NeutronMsg>>> {
    let addrs = get_contracts!(deps, config.factory_contract, puppeteer_contract);
    let pending_coin = deps
        .querier
        .query_balance(&env.contract.address, config.base_denom.to_string())?;

    if pending_coin.amount < config.min_ibc_transfer {
        return Ok(None);
    }

    let puppeteer_ica: drop_helpers::ica::IcaState = deps.querier.query_wasm_smart(
        addrs.puppeteer_contract,
        &drop_puppeteer_base::msg::QueryMsg::<Empty>::Ica {},
    )?;

    if let drop_helpers::ica::IcaState::Registered { ica_address, .. } = puppeteer_ica {
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
                denom: pending_coin.denom,
                amount: pending_coin.amount.u128(),
                real_amount: pending_coin.amount.u128(),
                recipient: ica_address.to_string(),
                reason: IBCTransferReason::Delegate,
            },
            ReplyMsg::IbcTransfer.to_reply_id(),
        )?;

        return Ok(Some(submsg));
    }

    Err(ContractError::IcaNotRegistered {})
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::peripheral_hook::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let addrs = get_contracts!(
        deps,
        config.factory_contract,
        core_contract,
        puppeteer_contract
    );

    ensure_eq!(
        info.sender,
        addrs.puppeteer_contract,
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
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(success_msg) => {
            if let drop_puppeteer_base::peripheral_hook::Transaction::Stake { amount } =
                success_msg.transaction
            {
                if let Transaction::Stake { .. } = transaction {
                    NON_STAKED_BALANCE
                        .update(deps.storage, |balance| StdResult::Ok(balance - amount))?;

                    TX_STATE.save(deps.storage, &TxState::default())?;
                }
            }
        }
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Error(error_msg) => {
            if let drop_puppeteer_base::peripheral_hook::Transaction::Stake { .. } =
                error_msg.transaction
            {
                TX_STATE.save(deps.storage, &TxState::default())?;
            }
        }
    }

    LAST_PUPPETEER_RESPONSE.save(deps.storage, &msg)?;

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(msg))?,
        funds: vec![],
    });

    let submessage = SubMsg::reply_on_error(hook_message, ReplyMsg::IbcTransfer.to_reply_id());

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    )
    .add_submessage(submessage))
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
        ReplyMsg::IbcTransfer | ReplyMsg::Bond => transaction_reply(deps),
        ReplyMsg::PuppeteerHookForward => Err(ContractError::MessageIsNotSupported {}),
    }
}

fn transaction_reply(deps: DepsMut) -> ContractResult<Response> {
    let mut tx_state: TxState = TX_STATE.load(deps.storage)?;

    tx_state.status = TxStateStatus::WaitingForAck;
    TX_STATE.save(deps.storage, &tx_state)?;

    if let Some(Transaction::IBCTransfer { amount, .. }) = tx_state.transaction {
        NON_STAKED_BALANCE.update(deps.storage, |balance| {
            StdResult::Ok(balance + Uint128::from(amount))
        })?;
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
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_error(deps, env, request, "Timeout".to_string()),
        _ => Err(ContractError::MessageIsNotSupported {}),
    }
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    details: String,
) -> ContractResult<Response<NeutronMsg>> {
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

    let attrs = vec![
        attr("action", "sudo_error"),
        attr("request_id", seq_id.to_string()),
    ];

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;

    if let Transaction::IBCTransfer { amount, .. } = transaction.clone() {
        NON_STAKED_BALANCE.update(deps.storage, |balance| {
            StdResult::Ok(balance - Uint128::from(amount))
        })?;
    }

    TX_STATE.save(deps.storage, &TxState::default())?;

    let config = CONFIG.load(deps.storage)?;
    let addrs = get_contracts!(deps, config.factory_contract, core_contract);

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
                transaction,
                details,
            },
        )))?,
        funds: vec![],
    });

    Ok(response("sudo-timeout", "puppeteer", attrs).add_message(hook_message))
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    request: RequestPacket,
    _data: Binary,
) -> ContractResult<Response<NeutronMsg>> {
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

    let channel_id = request
        .clone()
        .source_channel
        .ok_or_else(|| StdError::generic_err("source_channel not found"))?;
    let port_id = request
        .clone()
        .source_port
        .ok_or_else(|| StdError::generic_err("source_port not found"))?;

    let client_state = query_client_state(&deps.as_ref(), channel_id, port_id)?;

    let remote_height = client_state
        .identified_client_state
        .ok_or_else(|| StdError::generic_err("IBC client state identified_client_state not found"))?
        .client_state
        .latest_height
        .ok_or_else(|| StdError::generic_err("IBC client state latest_height not found"))?
        .revision_height;

    let attrs = vec![attr("action", "sudo_response")];

    TX_STATE.save(deps.storage, &TxState::default())?;

    let config = CONFIG.load(deps.storage)?;
    let addrs = get_contracts!(deps, config.factory_contract, core_contract);

    deps.api.debug(&format!(
        "WASMDEBUG: json: {request:?}",
        request = to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
            ResponseHookMsg::Success(ResponseHookSuccessMsg {
                transaction: transaction.clone(),
                local_height: env.block.height,
                remote_height: remote_height.u64(),
            },)
        ))?
    ));
    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
            ResponseHookMsg::Success(ResponseHookSuccessMsg {
                transaction: transaction.clone(),
                local_height: env.block.height,
                remote_height: remote_height.u64(),
            }),
        ))?,
        funds: vec![],
    });

    Ok(response("sudo-response", "puppeteer", attrs).add_message(hook_message))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let contract_version_metadata = cw2::get_contract_version(deps.storage)?;
    let storage_contract_name = contract_version_metadata.contract.as_str();
    if storage_contract_name != CONTRACT_NAME {
        return Err(ContractError::MigrationError {
            storage_contract_name: storage_contract_name.to_string(),
            contract_name: CONTRACT_NAME.to_string(),
        });
    }

    let storage_version: semver::Version = contract_version_metadata.version.parse()?;
    let version: semver::Version = CONTRACT_VERSION.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}
