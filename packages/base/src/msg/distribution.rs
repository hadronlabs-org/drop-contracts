use crate::error::distribution::{ContractError, ContractResult};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use std::collections::HashMap;

#[cw_serde]
pub struct Delegations {
    pub total_stake: Uint128,
    pub total_on_top: Uint128,
    pub total_weight: u64,
    pub delegations: Vec<Delegation>,
}

#[derive(Default)]
pub struct StakeChanges {
    changes: HashMap<String, Uint128>,
}

impl StakeChanges {
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    pub fn push(&mut self, addr: impl Into<String>, change: impl Into<Uint128>) {
        *self.changes.entry(addr.into()).or_insert(Uint128::zero()) += change.into()
    }

    pub fn into_vec(self) -> Vec<(String, Uint128)> {
        self.changes.into_iter().collect()
    }
}

impl Delegations {
    pub fn deposit_on_top(
        &mut self,
        stake_changes: &mut StakeChanges,
        deposit: &mut Uint128,
    ) -> ContractResult<()> {
        if self.total_on_top.is_zero() {
            return Ok(());
        }

        let undersatisfaction = self.delegations.iter().fold(Uint128::zero(), |acc, d| {
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
        self.total_stake += to_deposit;
        *deposit -= to_deposit;

        let mut total_stake = (self.total_on_top - undersatisfaction) + to_deposit;
        let stake_per_weight = Decimal::from_ratio(total_stake, self.total_on_top);
        for d in &mut self.delegations {
            let weight = Decimal::from_atomics(d.on_top, 0)?;
            let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

            if total_stake < ideal_stake {
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

    pub fn deposit_normal(
        self,
        stake_changes: &mut StakeChanges,
        mut deposit: Uint128,
    ) -> ContractResult<()> {
        if deposit.is_zero() {
            return Ok(());
        }

        let mut total_stake = self.excess() + deposit;
        let stake_per_weight = Decimal::from_ratio(total_stake, self.total_weight);
        for d in self.delegations {
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

    pub fn withdraw_normal(
        &mut self,
        stake_changes: &mut StakeChanges,
        withdraw: &mut Uint128,
    ) -> ContractResult<()> {
        if self.total_stake < *withdraw {
            return Err(ContractError::TooBigWithdraw {});
        }

        let excess = self.excess();
        let mut to_withdraw = *withdraw;
        if to_withdraw > excess {
            to_withdraw = excess;
        }
        *withdraw -= to_withdraw;
        self.total_stake -= to_withdraw;
        let mut total_stake = excess - to_withdraw;

        let stake_per_weight = Decimal::from_ratio(total_stake, self.total_weight);
        for d in &mut self.delegations {
            let weight = Decimal::from_atomics(d.weight, 0)?;
            let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil();

            if total_stake < ideal_stake {
                ideal_stake = total_stake;
            }
            total_stake -= ideal_stake;
            if ideal_stake >= (d.stake - d.on_top) || to_withdraw.is_zero() {
                continue;
            }

            let mut stake_change = (d.stake - d.on_top) - ideal_stake;
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

    pub fn withdraw_on_top(
        self,
        stake_changes: &mut StakeChanges,
        mut withdraw: Uint128,
    ) -> ContractResult<()> {
        if withdraw.is_zero() {
            return Ok(());
        }

        let mut total_stake = self.total_stake - withdraw;
        let stake_per_weight = Decimal::from_ratio(total_stake, self.total_on_top);

        for d in self.delegations {
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

    fn excess(&self) -> Uint128 {
        self.delegations.iter().fold(Uint128::zero(), |acc, d| {
            assert!(d.stake >= d.on_top);
            acc + (d.stake - d.on_top)
        })
    }
}

#[cw_serde]
pub struct Delegation {
    pub valoper_address: String,
    pub stake: Uint128,
    pub on_top: Uint128,
    pub weight: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<(String, Uint128)>)]
    CalcDeposit {
        deposit: Uint128,
        delegations: Delegations,
    },
    #[returns(Vec<(String, Uint128)>)]
    CalcWithdraw {
        withdraw: Uint128,
        delegations: Delegations,
    },
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}
