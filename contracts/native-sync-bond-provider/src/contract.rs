use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Attribute, Coin, CosmosMsg, Decimal, Deps, Uint128,
    WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_staking_base::{
    msg::core::ExecuteMsg as CoreExecuteMsg,
    msg::native_sync_bond_provider::{
        ConfigOptional, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    state::native_sync_bond_provider::{Config, CONFIG},
};

use drop_puppeteer_base::peripheral_hook::ReceiverExecuteMsg;
use drop_staking_base::error::native_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::core::LastPuppeteerResponse;
use drop_staking_base::state::native_bond_provider::{TxState, LAST_PUPPETEER_RESPONSE};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;

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
    };
    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("factory_contract", factory_contract.into_string())],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::CanBond { denom } => query_can_bond(denom),
        QueryMsg::CanProcessOnIdle {} => {
            Ok(to_json_binary(&query_can_process_on_idle(deps, &env)?)?)
        }
        QueryMsg::TokensAmount {
            coin,
            exchange_rate,
        } => query_token_amount(coin, exchange_rate),
        QueryMsg::AsyncTokensAmount {} => query_async_tokens_amount(deps, env),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::TxState {} => query_tx_state(deps, env),
        QueryMsg::LastPuppeteerResponse {} => Ok(to_json_binary(&LastPuppeteerResponse {
            response: LAST_PUPPETEER_RESPONSE.may_load(deps.storage)?,
        })?),
        QueryMsg::CanBeRemoved {} => todo!(),
    }
}

fn query_tx_state(_deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    Ok(to_json_binary(&TxState::default())?)
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_non_staked_balance(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let balance = deps
        .querier
        .query_balance(env.contract.address, LOCAL_DENOM)?
        .amount;
    Ok(to_json_binary(&(balance))?)
}

fn query_async_tokens_amount(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    query_non_staked_balance(deps, env)
}

fn query_can_bond(denom: String) -> ContractResult<Binary> {
    Ok(to_json_binary(&can_bond(LOCAL_DENOM.to_string(), denom))?)
}

fn query_can_process_on_idle(deps: Deps<NeutronQuery>, env: &Env) -> ContractResult<bool> {
    let non_staked_balance = deps
        .querier
        .query_balance(&env.contract.address, LOCAL_DENOM.to_string())?
        .amount;

    ensure!(
        !non_staked_balance.is_zero(),
        ContractError::NotEnoughToProcessIdle {
            min_stake_amount: Uint128::new(1),
            min_ibc_transfer: Uint128::new(0),
            non_staked_balance,
            pending_coins: Uint128::new(0),
        }
    );

    Ok(true)
}

fn query_token_amount(coin: Coin, exchange_rate: Decimal) -> ContractResult<Binary> {
    if can_bond(LOCAL_DENOM.to_string(), coin.denom) {
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
        ExecuteMsg::Bond {} => execute_bond(info),
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

    CONFIG.save(deps.storage, &state)?;

    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_bond(info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    let Coin { amount, denom } = cw_utils::one_coin(&info)?;

    if denom != *LOCAL_DENOM {
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
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, core_contract);
    ensure_eq!(
        info.sender,
        deps.api.addr_validate(&addrs.core_contract)?,
        ContractError::Unauthorized {}
    );
    query_can_process_on_idle(deps.as_ref(), &env)?;

    let attrs = vec![attr("action", "process_on_idle")];
    let mut messages: Vec<CosmosMsg<NeutronMsg>> = vec![];

    if let Some(delegate_msg) = get_delegation_msg(deps.branch(), &env, &config)? {
        messages.push(delegate_msg);
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
    let non_staked_balance = deps
        .querier
        .query_balance(&env.contract.address, LOCAL_DENOM.to_string())?
        .amount;

    if non_staked_balance.is_zero() {
        return Ok(None);
    }

    let addrs = drop_helpers::get_contracts!(
        deps,
        config.factory_contract,
        strategy_contract,
        puppeteer_contract
    );

    let to_delegate: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        &addrs.strategy_contract,
        &drop_staking_base::msg::strategy::QueryMsg::CalcDeposit {
            deposit: non_staked_balance,
        },
    )?;
    let puppeteer_delegation_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.puppeteer_contract,
        msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
            items: to_delegate,
            reply_to: env.contract.address.to_string(),
        })?,
        funds: vec![Coin {
            denom: LOCAL_DENOM.to_string(),
            amount: non_staked_balance,
        }],
    });

    Ok(Some(puppeteer_delegation_msg))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::peripheral_hook::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let addrs = drop_helpers::get_contracts!(
        deps,
        config.factory_contract,
        puppeteer_contract,
        core_contract
    );
    ensure_eq!(
        info.sender,
        deps.api.addr_validate(&addrs.puppeteer_contract)?,
        ContractError::Unauthorized {}
    );

    LAST_PUPPETEER_RESPONSE.save(deps.storage, &msg)?;
    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(msg))?,
        funds: vec![],
    });

    let tick_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.core_contract.to_string(),
        msg: to_json_binary(&CoreExecuteMsg::Tick {})?,
        funds: vec![],
    });

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    )
    .add_message(hook_message)
    .add_message(tick_message))
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
