use std::marker::PhantomData;

use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{ACCOUNTS, PREFIX},
};
use cosmwasm_std::{
    attr, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

const DEFAULT_LIMIT: u32 = 10;
const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let attrs = vec![attr("action", "instantiate"), (attr("prefix", &msg.prefix))];
    PREFIX.save(deps.storage, &msg.prefix)?;
    Ok(response("instantiate", "hook-tester", attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::GetOne { address } => query_one(deps, address),
        QueryMsg::GetAll { start_after, limit } => query_all(deps, start_after, limit),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Link { address } => execute_link(deps, env, info, address),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
    }
}

fn query_one(deps: Deps, address: String) -> ContractResult<Binary> {
    let one = ACCOUNTS.may_load(deps.storage, address)?;
    Ok(to_json_binary(&one)?)
}

fn query_all(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> ContractResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;
    let all = ACCOUNTS
        .range(
            deps.storage,
            start_after.map(|x| cw_storage_plus::Bound::Inclusive((x, PhantomData))),
            None,
            cosmwasm_std::Order::Ascending,
        )
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
    Ok(to_json_binary(&all)?)
}

fn execute_link(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> ContractResult<Response> {
    let prefix = PREFIX.load(deps.storage)?;
    if address.starts_with(&prefix) {
        return Err(ContractError::WrongPrefix {});
    }
    bech32::decode(&address).map_err(|_| ContractError::WrongAddress {})?;
    ACCOUNTS.save(deps.storage, info.sender.to_string(), &address)?;
    let attrs = vec![
        (attr("sender".to_string(), info.sender.to_string())),
        (attr("address".to_string(), address.to_string())),
    ];
    Ok(response("execute-link", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}

pub fn response<A: Into<cosmwasm_std::Attribute>, T>(
    ty: &str,
    contract_name: &str,
    attrs: impl IntoIterator<Item = A>,
) -> Response<T> {
    Response::<T>::new().add_event(
        cosmwasm_std::Event::new(format!("{}-{}", contract_name, ty)).add_attributes(attrs),
    )
}
