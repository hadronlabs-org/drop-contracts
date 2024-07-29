use cosmwasm_std::{entry_point, to_json_binary, BankMsg, Coin, CosmosMsg, Deps, Uint128};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use drop_staking_base::{
    error::splitter::{ContractError, ContractResult},
    msg::splitter::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::splitter::{Config, CONFIG},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    CONFIG.save(deps.storage, &msg.config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            to_json_binary(&CONFIG.load(deps.storage)?).map_err(ContractError::from)
        }
        QueryMsg::Ownership {} => query_ownership(deps),
    }
}

pub fn query_ownership(deps: Deps) -> ContractResult<Binary> {
    let ownership = cw_ownable::get_ownership(deps.storage)?;
    to_json_binary(&ownership).map_err(ContractError::from)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Distribute {} => execute_distribute(deps, env, info),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_config: Config,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::default())
}

pub fn execute_distribute(deps: DepsMut, env: Env, _info: MessageInfo) -> ContractResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let total_share = config
        .receivers
        .iter()
        .fold(Uint128::zero(), |acc, (_, share)| acc + share);
    if total_share.is_zero() {
        return Err(ContractError::NoShares {});
    }
    let balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), config.denom.to_string())?
        .amount;
    if balance < total_share {
        return Err(ContractError::InsufficientFunds {});
    }
    let k = balance / total_share;
    let messages = config
        .receivers
        .iter()
        .map(|(receiver, share)| {
            CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![Coin::new((share * k).u128(), config.denom.to_string())],
            })
        })
        .collect::<Vec<_>>();
    Ok(Response::default().add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
