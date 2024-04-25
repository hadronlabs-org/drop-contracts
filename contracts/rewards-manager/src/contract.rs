use cosmwasm_std::{
    attr, ensure, entry_point, to_json_binary, Attribute, CosmosMsg, Deps, Order, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use drop_helpers::answer::response;
use drop_helpers::pause::{assert_paused, is_paused, set_pause, unpause, PauseInfoResponse};
use drop_staking_base::error::rewards_manager::{ContractError, ContractResult};
use drop_staking_base::msg::rewards_manager::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::rewards_manager::{HandlerConfig, REWARDS_HANDLERS};

use drop_staking_base::msg::reward_handler::HandlerExecuteMsg;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let owner = deps.api.addr_validate(&msg.owner)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_ref()))?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("owner", msg.owner)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::Handlers {} => query_handlers(deps, env),
        QueryMsg::PauseInfo {} => query_pause_info(deps),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let owner = cw_ownable::get_ownership(deps.storage)?;

    to_json_binary(&ConfigResponse {
        owner: owner
            .owner
            .map(|addr| addr.into_string())
            .unwrap_or_default(),
    })
}

fn query_pause_info(deps: Deps) -> StdResult<Binary> {
    if is_paused(deps.storage) {
        to_json_binary(&PauseInfoResponse::Paused {})
    } else {
        to_json_binary(&PauseInfoResponse::Unpaused {})
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => exec_update_config(deps, info, owner),
        ExecuteMsg::AddHandler { config } => exec_add_handler(deps, info, config),
        ExecuteMsg::RemoveHandler { denom } => exec_remove_handler(deps, info, denom),
        ExecuteMsg::ExchangeRewards { denoms } => exec_exchange_rewards(deps, env, info, denoms),
        ExecuteMsg::Pause {} => exec_pause(deps, info),
        ExecuteMsg::Unpause {} => exec_unpause(deps, info),
    }
}

fn exec_pause(deps: DepsMut, info: MessageInfo) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    set_pause(deps.storage)?;

    Ok(response(
        "exec_pause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn exec_unpause(deps: DepsMut, info: MessageInfo) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    unpause(deps.storage)?;

    Ok(response(
        "exec_unpause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn exec_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(owner) = owner {
        let owner = deps.api.addr_validate(&owner)?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_ref()))?;
        attrs.push(attr("owner", owner))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn exec_add_handler(
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

fn exec_exchange_rewards(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    denoms: Vec<String>,
) -> ContractResult<Response> {
    assert_paused(deps.storage)?;

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
