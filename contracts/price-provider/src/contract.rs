use cosmwasm_std::{entry_point, to_json_binary, Decimal, Deps};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use drop_helpers::answer::response;
use drop_staking_base::error::price_provider::{ContractError, ContractResult};
use drop_staking_base::msg::price_provider::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use drop_staking_base::state::price_provider::PAIRS_PRICES;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps
        .api
        .addr_validate(&msg.owner.unwrap_or(info.sender.to_string()))?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Price { pair } => query_price(deps, pair),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
    }
}

fn query_price(deps: Deps, pair: (String, String)) -> ContractResult<Binary> {
    let price =
        PAIRS_PRICES
            .load(deps.storage, &pair)
            .map_err(|e| ContractError::PairNotFound {
                details: e.to_string(),
            })?;

    to_json_binary(&price).map_err(ContractError::Std)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::RemovePair { pair } => execute_remove_pair(deps, env, info, pair),
        ExecuteMsg::SetPrice { pair, price } => execute_set_price(deps, env, info, pair, price),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
    }
}

fn execute_set_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pair: (String, String),
    price: Decimal,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    PAIRS_PRICES.save(deps.storage, &pair, &price)?;
    Ok(response::<(&str, &str), _>(
        "execute-set-price",
        CONTRACT_NAME,
        [],
    ))
}

fn execute_remove_pair(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pair: (String, String),
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    PAIRS_PRICES.remove(deps.storage, &pair);
    Ok(response::<(&str, &str), _>(
        "execute-remove-pair",
        CONTRACT_NAME,
        [],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
