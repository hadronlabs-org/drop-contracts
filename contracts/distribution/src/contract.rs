use cosmwasm_std::{
    to_json_binary, Attribute, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, Uint128,
};
use drop_helpers::answer::response;
use drop_staking_base::msg::distribution::Delegation;
use drop_staking_base::{
    error::distribution::{ContractError, ContractResult},
    msg::distribution::{Delegations, InstantiateMsg, MigrateMsg, QueryMsg},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::collections::HashMap;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
struct StakeChanges {
    changes: HashMap<String, Uint128>,
}

impl StakeChanges {
    fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    fn push(&mut self, addr: impl Into<String>, change: impl Into<Uint128>) {
        *self.changes.entry(addr.into()).or_insert(Uint128::zero()) += change.into()
    }

    fn into_vec(self) -> Vec<(String, Uint128)> {
        self.changes.into_iter().collect()
    }
}

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
    calc_withdraw_normal(&mut delegations, &mut stake_changes, &mut withdraw)?;
    calc_withdraw_on_top(delegations, &mut stake_changes, withdraw)?;
    Ok(stake_changes.into_vec())
}

/// Calculates the ideal distribution of stake among the given delegations.
pub fn calc_deposit(
    mut deposit: Uint128,
    mut delegations: Delegations,
) -> ContractResult<Vec<(String, Uint128)>> {
    let mut stake_changes = StakeChanges::new();
    calc_deposit_on_top(&mut delegations, &mut stake_changes, &mut deposit)?;
    calc_deposit_normal(delegations, &mut stake_changes, deposit)?;
    Ok(stake_changes.into_vec())
}

fn calc_excess(delegation: &Delegation) -> Uint128 {
    if delegation.stake >= delegation.on_top {
        delegation.stake - delegation.on_top
    } else {
        Uint128::zero()
    }
}

fn calc_withdraw_normal(
    delegations: &mut Delegations,
    stake_changes: &mut StakeChanges,
    withdraw: &mut Uint128,
) -> ContractResult<()> {
    if delegations.total_stake < *withdraw {
        return Err(ContractError::TooBigWithdraw {});
    }

    let excess = delegations
        .delegations
        .iter()
        .fold(Uint128::zero(), |acc, d| acc + calc_excess(d));
    let mut to_withdraw = *withdraw;
    if to_withdraw > excess {
        to_withdraw = excess;
    }
    *withdraw -= to_withdraw;
    delegations.total_stake -= to_withdraw;
    let mut total_stake = excess - to_withdraw;

    let stake_per_weight = Decimal::from_ratio(total_stake, delegations.total_weight);
    for d in &mut delegations.delegations {
        let weight = Decimal::from_atomics(d.weight, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if total_stake < ideal_stake {
            ideal_stake = total_stake;
        }
        total_stake -= ideal_stake;
        if ideal_stake >= calc_excess(d) || to_withdraw.is_zero() {
            continue;
        }

        let mut stake_change = calc_excess(d) - ideal_stake;
        if to_withdraw < stake_change {
            stake_change = to_withdraw;
        }
        to_withdraw -= stake_change;
        d.stake -= stake_change;
        assert!(d.stake >= d.on_top);
        stake_changes.push(&d.valoper_address, stake_change);
    }

    assert!(to_withdraw.is_zero());
    Ok(())
}

fn calc_withdraw_on_top(
    delegations: Delegations,
    stake_changes: &mut StakeChanges,
    mut withdraw: Uint128,
) -> ContractResult<()> {
    if withdraw.is_zero() {
        return Ok(());
    }

    let mut total_stake = delegations.total_stake - withdraw;
    let stake_per_weight = Decimal::from_ratio(total_stake, delegations.total_on_top);

    for d in delegations.delegations {
        assert!(d.stake <= d.on_top);
        let weight = Decimal::from_atomics(d.on_top, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if total_stake < ideal_stake {
            ideal_stake = total_stake;
        }
        total_stake -= ideal_stake;
        if ideal_stake >= d.stake || withdraw.is_zero() {
            continue;
        }

        let mut stake_change = d.stake - ideal_stake;
        if withdraw < stake_change {
            stake_change = withdraw;
        }
        withdraw -= stake_change;
        stake_changes.push(d.valoper_address, stake_change);
    }

    assert!(withdraw.is_zero());
    Ok(())
}

fn calc_deposit_on_top(
    delegations: &mut Delegations,
    stake_changes: &mut StakeChanges,
    deposit: &mut Uint128,
) -> ContractResult<()> {
    if delegations.total_on_top.is_zero() {
        return Ok(());
    }

    let undersatisfaction = delegations
        .delegations
        .iter()
        .fold(Uint128::zero(), |acc, d| {
            if d.on_top > d.stake {
                acc + (d.on_top - d.stake)
            } else {
                acc
            }
        });

    let mut to_deposit = *deposit;
    if to_deposit > undersatisfaction {
        to_deposit = undersatisfaction;
    }
    delegations.total_stake += to_deposit;
    *deposit -= to_deposit;

    let mut total_stake = (delegations.total_on_top - undersatisfaction) + to_deposit;
    let stake_per_weight = Decimal::from_ratio(total_stake, delegations.total_on_top);
    for d in &mut delegations.delegations {
        let weight = Decimal::from_atomics(d.on_top, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if ideal_stake >= total_stake {
            ideal_stake = total_stake;
        }
        total_stake -= ideal_stake;
        if ideal_stake <= d.stake || to_deposit.is_zero() {
            continue;
        }

        let mut stake_change = ideal_stake - d.stake;
        if to_deposit < stake_change {
            stake_change = to_deposit;
        }
        to_deposit -= stake_change;
        d.stake += stake_change;
        assert!(d.stake <= d.on_top);
        stake_changes.push(&d.valoper_address, stake_change)
    }

    assert!(to_deposit.is_zero());
    Ok(())
}

fn calc_deposit_normal(
    delegations: Delegations,
    stake_changes: &mut StakeChanges,
    mut deposit: Uint128,
) -> ContractResult<()> {
    if deposit.is_zero() {
        return Ok(());
    }

    let excess = delegations
        .delegations
        .iter()
        .fold(Uint128::zero(), |acc, d| acc + calc_excess(d));
    let mut total_stake = excess + deposit;
    let stake_per_weight = Decimal::from_ratio(total_stake, delegations.total_weight);
    for d in delegations.delegations {
        assert!(d.stake >= d.on_top);
        let weight = Decimal::from_atomics(d.weight, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if total_stake < ideal_stake {
            ideal_stake = total_stake;
        }
        total_stake -= ideal_stake;
        if ideal_stake <= (d.stake - d.on_top) || deposit.is_zero() {
            continue;
        }

        let mut stake_change = ideal_stake - (d.stake - d.on_top);
        if deposit < stake_change {
            stake_change = deposit;
        }
        deposit -= stake_change;
        stake_changes.push(d.valoper_address, stake_change)
    }

    assert!(deposit.is_zero());
    Ok(())
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
