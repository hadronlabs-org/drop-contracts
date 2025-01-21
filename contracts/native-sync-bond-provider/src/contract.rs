use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Attribute, BankMsg, Coin, CosmosMsg, Decimal, Deps,
    StdResult, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_staking_base::{
    msg::native_sync_bond_provider::{
        ConfigOptional, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    state::native_sync_bond_provider::{Config, CONFIG, NON_STAKED_BALANCE},
};

use drop_puppeteer_base::peripheral_hook::ReceiverExecuteMsg;
use drop_staking_base::error::native_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::core::LastPuppeteerResponse;
use drop_staking_base::state::native_bond_provider::{TxState, LAST_PUPPETEER_RESPONSE};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;

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
    };
    CONFIG.save(deps.storage, config)?;

    NON_STAKED_BALANCE.save(deps.storage, &Uint128::zero())?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("puppeteer_contract", puppeteer_contract.into_string()),
            attr("core_contract", core_contract.into_string()),
            attr("strategy_contract", strategy_contract.into_string()),
            attr("base_denom", msg.base_denom),
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
    }
}

fn query_tx_state(_deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    Ok(to_json_binary(&TxState::default())?)
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
    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let pending_coin = deps
        .querier
        .query_balance(&env.contract.address, config.base_denom.to_string())?;

    ensure!(
        !pending_coin.amount.is_zero() || !non_staked_balance.is_zero(),
        ContractError::NotEnoughToProcessIdle {
            min_stake_amount: Uint128::new(1),
            min_ibc_transfer: Uint128::new(0),
            non_staked_balance,
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

    let message = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.puppeteer_contract.to_string(),
        amount: vec![Coin {
            denom: config.base_denom,
            amount,
        }],
    });

    NON_STAKED_BALANCE.update(deps.storage, |balance| StdResult::Ok(balance + amount))?;

    Ok(response(
        "bond",
        CONTRACT_NAME,
        [attr_coin("received_funds", amount.to_string(), denom)],
    )
    .add_message(message))
}

fn execute_process_on_idle(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.core_contract,
        ContractError::Unauthorized {}
    );

    query_can_process_on_idle(deps.as_ref(), &env, &config)?;

    let attrs = vec![attr("action", "process_on_idle")];
    let mut messages: Vec<CosmosMsg<NeutronMsg>> = vec![];

    if let Some(lsm_msg) = get_delegation_msg(deps.branch(), &env, &config)? {
        messages.push(lsm_msg);
    }

    Ok(
        response("process_on_idle", CONTRACT_NAME, Vec::<Attribute>::new())
            .add_messages(messages)
            .add_attributes(attrs),
    )
}

fn get_delegation_msg(
    deps: DepsMut<NeutronQuery>,
    env: &Env,
    config: &Config,
) -> ContractResult<Option<CosmosMsg<NeutronMsg>>> {
    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;

    if non_staked_balance.is_zero() {
        return Ok(None);
    }

    let to_delegate: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        &config.strategy_contract,
        &drop_staking_base::msg::strategy::QueryMsg::CalcDeposit {
            deposit: non_staked_balance,
        },
    )?;
    let puppeteer_delegation_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract.to_string(),
        msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
            items: to_delegate,
            reply_to: env.contract.address.to_string(),
        })?,
        funds: vec![],
    });

    NON_STAKED_BALANCE.save(deps.storage, &Uint128::zero())?;

    Ok(Some(puppeteer_delegation_msg))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::peripheral_hook::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );

    LAST_PUPPETEER_RESPONSE.save(deps.storage, &msg)?;

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(msg))?,
        funds: vec![],
    });

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    )
    .add_message(hook_message))
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
