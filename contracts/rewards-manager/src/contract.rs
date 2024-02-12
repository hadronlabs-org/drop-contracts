use cosmwasm_std::{attr, entry_point, to_json_binary, Attribute, CosmosMsg, Deps, Order, WasmMsg};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::response;
use lido_staking_base::error::rewards_manager::ContractResult;
use lido_staking_base::msg::rewards_manager::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use lido_staking_base::state::rewards_manager::{HandlerConfig, CORE_ADDRESS, REWARDS_HANDLERS};

use lido_staking_base::msg::reward_handler::HandlerExecuteMsg;

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let core = deps.api.addr_validate(&msg.core_address)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core.as_ref()))?;
    CORE_ADDRESS.save(deps.storage, &core)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("core_address", msg.core_address)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::Handlers {} => query_handlers(deps, env),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();

    to_json_binary(&ConfigResponse { core_address })
}

fn query_handlers(deps: Deps, _env: Env) -> StdResult<Binary> {
    let handlers: StdResult<Vec<_>> = REWARDS_HANDLERS
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect();

    let handlers = handlers.unwrap_or_default();

    to_json_binary(&handlers)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { core_address } => exec_update_config(deps, info, core_address),
        ExecuteMsg::AddHandler { config } => exec_add_handler(deps, info, config),
        ExecuteMsg::RemoveHandler { denom } => exec_remove_handler(deps, info, denom),
        ExecuteMsg::ExchangeRewards {} => exec_exchange_rewards(deps, env, info),
    }
}

fn exec_update_config(
    deps: DepsMut,
    info: MessageInfo,
    core_address: Option<String>,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        CORE_ADDRESS.save(deps.storage, &core_address)?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(core_address.as_ref()))?;
        attrs.push(attr("core_address", core_address))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn exec_add_handler(
    deps: DepsMut,
    info: MessageInfo,
    config: HandlerConfig,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    REWARDS_HANDLERS.save(deps.storage, config.denom.clone(), &config)?;

    Ok(response(
        "add_handler",
        CONTRACT_NAME,
        [
            attr("denom", config.denom),
            attr("address", config.address),
            attr("min_rewards", config.min_rewards.to_string()),
        ],
    ))
}

fn exec_remove_handler(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    REWARDS_HANDLERS.remove(deps.storage, denom.clone());

    Ok(response(
        "remove_handler",
        CONTRACT_NAME,
        [attr("denom", denom)],
    ))
}

fn exec_exchange_rewards(deps: DepsMut, env: Env, _info: MessageInfo) -> ContractResult<Response> {
    let balances = deps.querier.query_all_balances(env.contract.address)?;

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut attrs: Vec<Attribute> = Vec::new();

    for balance in &balances {
        let denom = balance.denom.clone();
        let amount = balance.amount;

        if REWARDS_HANDLERS.has(deps.storage, denom.clone()) {
            let handler = REWARDS_HANDLERS.load(deps.storage, denom.clone())?;

            if amount < handler.min_rewards {
                continue;
            }

            let exchange_rewards_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: handler.address,
                msg: to_json_binary(&HandlerExecuteMsg::Exchange {})?,
                funds: vec![balance.clone()],
            });

            messages.push(exchange_rewards_msg);
            attrs.push(attr("denom", denom));
        }
    }

    Ok(response(
        "exchange_rewards",
        CONTRACT_NAME,
        [attr("total_denoms", balances.len().to_string())],
    )
    .add_messages(messages)
    .add_attributes(attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
