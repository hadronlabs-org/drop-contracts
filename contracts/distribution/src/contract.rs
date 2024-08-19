use cosmwasm_std::{
    to_json_binary, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Response, Uint128,
};
use drop_helpers::answer::response;
use drop_staking_base::{
    error::distribution::ContractResult,
    msg::distribution::{Delegations, InstantiateMsg, MigrateMsg, QueryMsg, StakeChanges},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let empty_attr: Vec<Attribute> = Vec::new();
    Ok(response("instantiate", CONTRACT_NAME, empty_attr))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::CalcDeposit {
            deposit,
            delegations,
        } => Ok(to_json_binary(&calc_deposit(deposit, delegations)?)?),
        QueryMsg::CalcWithdraw {
            withdraw,
            delegations,
        } => Ok(to_json_binary(&calc_withdraw(withdraw, delegations)?)?),
    }
}

/// Calculates the ideal withdrawal of stake among the given withdraw amount.
pub fn calc_withdraw(
    mut withdraw: Uint128,
    mut delegations: Delegations,
) -> ContractResult<Vec<(String, Uint128)>> {
    let mut stake_changes = StakeChanges::new();
    delegations.withdraw_normal(&mut stake_changes, &mut withdraw)?;
    delegations.withdraw_on_top(&mut stake_changes, withdraw)?;
    Ok(stake_changes.into_vec())
}

/// Calculates the ideal distribution of stake among the given delegations.
pub fn calc_deposit(
    mut deposit: Uint128,
    mut delegations: Delegations,
) -> ContractResult<Vec<(String, Uint128)>> {
    let mut stake_changes = StakeChanges::new();
    delegations.deposit_on_top(&mut stake_changes, &mut deposit)?;
    delegations.deposit_normal(&mut stake_changes, deposit)?;
    Ok(stake_changes.into_vec())
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
