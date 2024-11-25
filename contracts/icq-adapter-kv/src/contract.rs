use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, WasmMsg,
};
use drop_helpers::answer::response;
use drop_staking_base::msg::icq_router::{BalancesData, DelegationsData};
use drop_staking_base::state::icq_adapter::{Config, ConfigOptional, CONFIG};
use drop_staking_base::{
    error::icq_adapter::{ContractError, ContractResult},
    msg::icq_adapter::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use std::{env, vec};

use crate::msg::Options;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg<Options>,
) -> ContractResult<Response<NeutronMsg>> {
    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;
    let router = deps.api.addr_validate(&msg.router)?;
    let attrs = vec![
        attr("action", "instantiate"),
        attr("router", router.to_string()),
        attr("owner", owner),
    ];
    CONFIG.save(deps.storage, &Config { router })?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Balances {} => query_balance(deps),
        QueryMsg::Delegations {} => query_delegations(deps),
        QueryMsg::NonNativeRewardsBalances {} => query_non_native_rewards_balances(deps),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => Ok(to_json_binary(&CONFIG.load(deps.storage)?)?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateValidators { validators } => {
            update_validators(deps, info, env, validators)
        }
        ExecuteMsg::UpdateConfig { new_config } => update_config(deps, info, new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            let attrs = vec![attr("action", "update_ownership")];
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response("update_ownership", CONTRACT_NAME, attrs))
        }
        ExecuteMsg::UpdateBalances { balances } => update_balances(deps, info, env, balances),
        ExecuteMsg::UpdateDelegations { delegations } => {
            update_delegations(deps, info, env, delegations)
        }
    }
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

fn query_balance(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let balance = BALANCES.load(deps.storage)?;
    Ok(to_json_binary(&balance)?)
}

fn query_delegations(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let delegations = DELEGATIONS.load(deps.storage)?;
    Ok(to_json_binary(&delegations)?)
}

fn query_non_native_rewards_balances(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let balances = NON_NATIVE_REWARD_BALANCES.load(deps.storage)?;
    Ok(to_json_binary(&balances)?)
}

fn update_balances(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    _env: Env,
    balances: BalancesData,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, config.adapter, ContractError::Unauthorized {});
    let old_data = BALANCES.load(deps.storage)?;
    ensure!(
        balances.remote_height > old_data.remote_height,
        ContractError::OutdatedData
    );
    BALANCES.save(deps.storage, &balances)?;
    Ok(Response::new())
}

fn update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut config = CONFIG.load(deps.storage)?;
    if let Some(adapter) = new_config.adapter {
        config.adapter = deps.api.addr_validate(&adapter)?;
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

fn update_delegations(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    _env: Env,
    delegations: DelegationsData,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, config.adapter, ContractError::Unauthorized {});
    let old_data = DELEGATIONS.load(deps.storage)?;
    ensure!(
        delegations.remote_height > old_data.remote_height,
        ContractError::OutdatedData
    );
    DELEGATIONS.save(deps.storage, &delegations)?;
    Ok(Response::new())
}

fn update_validators(
    deps: DepsMut<NeutronQuery>,
    _info: MessageInfo,
    _env: Env,
    validators: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let res = response(
        "update_validators",
        CONTRACT_NAME,
        [attr("validators", format!("{:?}", validators))],
    );
    Ok(res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.adapter.to_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::icq_adapter::ExecuteMsg::UpdateValidatorSet { validators },
        )?,
        funds: vec![],
    })))
}
