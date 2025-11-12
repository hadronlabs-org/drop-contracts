use cosmwasm_std::{to_json_binary, Decimal, Deps};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use drop_helpers::answer::response;
use drop_staking_base::error::price_provider::{ContractError, ContractResult};
use drop_staking_base::msg::price_provider::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use drop_staking_base::state::price_provider::PRICES;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps
        .api
        .addr_validate(&msg.owner.unwrap_or(info.sender.to_string()))?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Price { denom } => query_price(deps, denom),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
    }
}

fn query_price(deps: Deps, denom: String) -> ContractResult<Binary> {
    let price = PRICES
        .load(deps.storage, &denom)
        .map_err(|e| ContractError::DenomNotFound {
            details: e.to_string(),
        })?;

    to_json_binary(&price).map_err(ContractError::Std)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::RemoveDenom { denom } => execute_remove_denom(deps, env, info, denom),
        ExecuteMsg::SetPrice { denom, price } => execute_set_price(deps, env, info, denom, price),
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
    denom: String,
    price: Decimal,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    PRICES.save(deps.storage, &denom, &price)?;
    Ok(response::<(&str, &str), _>(
        "execute-set-price",
        CONTRACT_NAME,
        [],
    ))
}

fn execute_remove_denom(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    PRICES.remove(deps.storage, &denom);
    Ok(response::<(&str, &str), _>(
        "execute-remove-denom",
        CONTRACT_NAME,
        [],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
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
