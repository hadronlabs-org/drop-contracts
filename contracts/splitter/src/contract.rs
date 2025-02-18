use cosmwasm_std::{attr, entry_point, to_json_binary, BankMsg, Coin, CosmosMsg, Deps, Uint128};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use drop_helpers::answer::response;
use drop_staking_base::{
    error::splitter::{ContractError, ContractResult},
    msg::splitter::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::splitter::{Config, CONFIG},
};

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    verify_config(deps.as_ref(), &msg.config)?;
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
    verify_config(deps.as_ref(), &new_config)?;
    CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::default())
}

pub fn execute_distribute(deps: DepsMut, env: Env, _info: MessageInfo) -> ContractResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut attrs = vec![];
    let total_share: Uint128 = config.receivers.iter().map(|(_, share)| share).sum();
    if total_share.is_zero() {
        return Err(ContractError::NoShares {});
    }
    attrs.push(attr("total_shares", total_share));
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
            let amount = (share * k).u128();
            attrs.push(attr(receiver, amount.to_string()));
            CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.to_string(),
                amount: vec![Coin::new(amount, config.denom.to_string())],
            })
        })
        .collect::<Vec<_>>();
    Ok(response("execute-distribute", CONTRACT_NAME, attrs).add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

fn verify_config(deps: Deps, config: &Config) -> ContractResult<()> {
    for (receiver, weight) in config.receivers.iter() {
        deps.api.addr_validate(receiver)?;
        if weight.is_zero() {
            return Err(ContractError::ZeroShare {});
        }
    }
    Ok(())
}
