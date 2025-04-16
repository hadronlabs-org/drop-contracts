use cosmwasm_std::{
    to_json_binary, Attribute, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, Uint128,
};
use drop_helpers::answer::response;
use drop_staking_base::msg::distribution::Delegation;
use drop_staking_base::{
    error::distribution::{ContractError, ContractResult},
    msg::distribution::{InstantiateMsg, MigrateMsg, QueryMsg},
};
use neutron_sdk::bindings::msg::NeutronMsg;
use std::collections::HashMap;

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
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

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
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
    mut delegations: Vec<Delegation>,
) -> ContractResult<Vec<(String, Uint128)>> {
    let mut total_stake = delegations.iter().map(|delegation| delegation.stake).sum();
    let mut stake_changes = StakeChanges::new();
    delegations.sort_by_key(|delegation| {
        let normal_stake = calc_normal_stake(delegation);
        if delegation.weight == 0 || delegation.on_top.is_zero() || normal_stake.is_zero() {
            Decimal::MIN
        } else {
            Decimal::from_ratio(delegation.weight, normal_stake)
        }
    });

    calc_withdraw_normal(
        &mut delegations,
        &mut stake_changes,
        &mut withdraw,
        &mut total_stake,
    )?;
    calc_withdraw_on_top(delegations, &mut stake_changes, withdraw, total_stake)?;
    Ok(stake_changes.into_vec())
}

/// Calculates the ideal distribution of stake among the given delegations.
pub fn calc_deposit(
    mut deposit: Uint128,
    mut delegations: Vec<Delegation>,
) -> ContractResult<Vec<(String, Uint128)>> {
    let mut total_stake = delegations.iter().map(|delegation| delegation.stake).sum();
    let mut stake_changes = StakeChanges::new();
    delegations.sort_by_key(|delegation| {
        if delegation.stake >= delegation.on_top {
            Uint128::MAX
        } else {
            Uint128::MAX - (delegation.on_top - delegation.stake)
        }
    });

    calc_deposit_on_top(
        &mut delegations,
        &mut stake_changes,
        &mut deposit,
        &mut total_stake,
    )?;
    calc_deposit_normal(delegations, &mut stake_changes, deposit)?;
    Ok(stake_changes.into_vec())
}

fn calc_normal_stake(delegation: &Delegation) -> Uint128 {
    if delegation.stake >= delegation.on_top {
        delegation.stake - delegation.on_top
    } else {
        Uint128::zero()
    }
}

fn calc_withdraw_normal(
    delegations: &mut [Delegation],
    stake_changes: &mut StakeChanges,
    withdraw: &mut Uint128,
    total_stake: &mut Uint128,
) -> ContractResult<()> {
    if *total_stake < *withdraw {
        return Err(ContractError::TooBigWithdraw {});
    }

    let excess = delegations
        .iter()
        .fold(Uint128::zero(), |acc, d| acc + calc_normal_stake(d));
    let mut to_withdraw = *withdraw;
    if to_withdraw > excess {
        to_withdraw = excess;
    }
    *withdraw -= to_withdraw;
    *total_stake -= to_withdraw;
    let mut total_target = excess - to_withdraw;

    let stake_per_weight = Decimal::from_ratio(
        total_target,
        delegations
            .iter()
            .map(|delegation| delegation.weight)
            .sum::<u64>(),
    );
    for d in delegations {
        let weight = Decimal::from_atomics(d.weight, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if total_target < ideal_stake {
            ideal_stake = total_target;
        }
        total_target -= ideal_stake;
        if ideal_stake >= calc_normal_stake(d) || to_withdraw.is_zero() {
            continue;
        }

        let mut stake_change = calc_normal_stake(d) - ideal_stake;
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
    delegations: Vec<Delegation>,
    stake_changes: &mut StakeChanges,
    mut withdraw: Uint128,
    total_stake: Uint128,
) -> ContractResult<()> {
    if withdraw.is_zero() {
        return Ok(());
    }

    let mut total_target = total_stake - withdraw;
    let total_on_top = delegations
        .iter()
        .map(|delegation| delegation.on_top)
        .sum::<Uint128>();
    let stake_per_weight = Decimal::from_ratio(total_target, total_on_top);

    for d in delegations {
        assert!(d.stake <= d.on_top);
        let weight = Decimal::from_atomics(d.on_top, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if total_target < ideal_stake {
            ideal_stake = total_target;
        }
        total_target -= ideal_stake;
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
    delegations: &mut [Delegation],
    stake_changes: &mut StakeChanges,
    deposit: &mut Uint128,
    total_stake: &mut Uint128,
) -> ContractResult<()> {
    let total_on_top = delegations
        .iter()
        .map(|delegation| delegation.on_top)
        .sum::<Uint128>();
    if total_on_top.is_zero() {
        return Ok(());
    }

    let undersatisfaction = delegations.iter().fold(Uint128::zero(), |acc, d| {
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
    *total_stake += to_deposit;
    *deposit -= to_deposit;

    let mut total_target = (total_on_top - undersatisfaction) + to_deposit;
    let stake_per_weight = Decimal::from_ratio(total_target, total_on_top);
    for d in delegations {
        let weight = Decimal::from_atomics(d.on_top, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if ideal_stake >= total_target {
            ideal_stake = total_target;
        }
        total_target -= ideal_stake;
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
    delegations: Vec<Delegation>,
    stake_changes: &mut StakeChanges,
    mut deposit: Uint128,
) -> ContractResult<()> {
    if deposit.is_zero() {
        return Ok(());
    }

    let excess = delegations
        .iter()
        .fold(Uint128::zero(), |acc, d| acc + calc_normal_stake(d));
    let mut total_target = excess + deposit;
    let stake_per_weight = Decimal::from_ratio(
        total_target,
        delegations
            .iter()
            .map(|delegation| delegation.weight)
            .sum::<u64>(),
    );
    for d in delegations {
        assert!(d.stake >= d.on_top);
        let weight = Decimal::from_atomics(d.weight, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

        if total_target < ideal_stake {
            ideal_stake = total_target;
        }
        total_target -= ideal_stake;
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
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response<NeutronMsg>> {
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
