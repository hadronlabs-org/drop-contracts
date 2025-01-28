use cosmwasm_std::{attr, ensure, to_json_binary, Attribute, CosmosMsg, Deps, Order, WasmMsg};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::{answer::response, is_paused};
use drop_staking_base::error::rewards_manager::{ContractError, ContractResult};
use drop_staking_base::msg::reward_handler::HandlerExecuteMsg;
use drop_staking_base::msg::rewards_manager::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use drop_staking_base::state::rewards_manager::{HandlerConfig, Pause, PAUSE, REWARDS_HANDLERS};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = deps.api.addr_validate(&msg.owner)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_ref()))?;
    PAUSE.save(deps.storage, &Pause::default())?;
    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("owner", msg.owner)],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Handlers {} => query_handlers(deps, env),
        QueryMsg::Pause {} => Ok(to_json_binary(&PAUSE.load(deps.storage)?)?),
    }
}

fn query_handlers(deps: Deps, _env: Env) -> StdResult<Binary> {
    let handlers: StdResult<Vec<_>> = REWARDS_HANDLERS
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect();

    let handlers = handlers.unwrap_or_default();

    to_json_binary(&handlers)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::AddHandler { config } => execute_add_handler(deps, info, config),
        ExecuteMsg::RemoveHandler { denom } => execute_remove_handler(deps, info, denom),
        ExecuteMsg::ExchangeRewards { denoms } => execute_exchange_rewards(deps, env, info, denoms),
        ExecuteMsg::SetPause { pause } => execute_set_pause(deps, info, pause),
    }
}

fn execute_set_pause(deps: DepsMut, info: MessageInfo, pause: Pause) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    pause.exchange_rewards.validate()?;
    PAUSE.save(deps.storage, &pause)?;
    let attrs = vec![("exchange_rewards", pause.exchange_rewards.to_string())];
    Ok(response("execute-set-pause", CONTRACT_NAME, attrs))
}

fn execute_add_handler(
    deps: DepsMut,
    info: MessageInfo,
    config: HandlerConfig,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    ensure!(
        !REWARDS_HANDLERS.has(deps.storage, config.denom.clone()),
        ContractError::DenomHandlerAlreadyExists
    );
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

fn execute_remove_handler(
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

fn execute_exchange_rewards(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    denoms: Vec<String>,
) -> ContractResult<Response> {
    if is_paused!(PAUSE, deps, env, exchange_rewards) {
        return Err(drop_helpers::pause::PauseError::Paused {}.into());
    }

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut attrs: Vec<Attribute> = Vec::new();
    let mut coins = vec![];
    ensure!(!denoms.is_empty(), ContractError::EmptyDenomsList);
    for denom in &denoms {
        let balance = deps
            .querier
            .query_balance(env.contract.address.to_string(), denom)?;
        coins.push(balance.clone());
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
        [
            attr("total_denoms", denoms.len().to_string()),
            attr("coins", format!("{:?}", coins)),
        ],
    )
    .add_messages(messages)
    .add_attributes(attrs))
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
        deps.storage.remove("paused".as_bytes());
        PAUSE.save(deps.storage, &Pause::default())?;
    }

    Ok(Response::new())
}
