use cosmwasm_std::{
    to_json_binary, Attribute, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, Uint128,
};
use drop_helpers::answer::response;
use drop_staking_base::error::distribution::{ContractError, ContractResult};
use drop_staking_base::msg::distribution::{Delegations, InstantiateMsg, MigrateMsg, QueryMsg};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::ops::Sub;

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
    withdraw: Uint128,
    delegations: Delegations,
) -> ContractResult<Vec<(String, Uint128)>> {
    if delegations.total < (withdraw + Uint128::from(delegations.delegations.len() as u128)) {
        return Err(ContractError::TooBigWithdraw {});
    }

    let total_stake = delegations.total - withdraw;
    let distribution = calc_deposit_distribution(
        total_stake,
        withdraw,
        delegations,
        |ideal_stake, current_stake| ideal_stake >= current_stake,
        |ideal_stake, current_stake| current_stake - ideal_stake,
    )?;

    Ok(distribution)
}

/// Calculates the ideal distribution of stake among the given delegations.
pub fn calc_deposit(
    deposit: Uint128,
    delegations: Delegations,
) -> ContractResult<Vec<(String, Uint128)>> {
    let total_stake = delegations.total + deposit;
    let distribution = calc_deposit_distribution(
        total_stake,
        deposit,
        delegations,
        |ideal_stake, current_stake| ideal_stake <= current_stake,
        |ideal_stake, current_stake| ideal_stake - current_stake,
    )?;

    Ok(distribution)
}

pub fn calc_deposit_distribution<C, D>(
    mut total_stake: Uint128,
    mut deposit: Uint128,
    delegations: Delegations,
    check_stake: C,
    calc_diff: D,
) -> ContractResult<Vec<(String, Uint128)>>
where
    C: Fn(Uint128, Uint128) -> bool,
    D: Fn(Uint128, Uint128) -> Uint128,
{
    let stake_per_weight = Decimal::from_ratio(total_stake, delegations.total_weight);

    // We need to distribute the deposit among all delegations (at least 1 token reservation required),
    // so we need to subtract amount of delegations (validators)
    deposit = deposit.sub(Uint128::from(delegations.delegations.len() as u128));
    let mut deposit_changes: Vec<(String, Uint128)> = Vec::new();
    for d in delegations.delegations {
        let weight = Decimal::from_atomics(d.weight, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil(); // ceil used to consume all available stake

        if total_stake < ideal_stake {
            ideal_stake = total_stake;
        }

        total_stake -= ideal_stake;

        if check_stake(ideal_stake, d.stake) || deposit.is_zero() {
            deposit_changes.push((d.valoper_address, Uint128::one()));
            continue;
        }

        // We need to add one because we already subtracted it before but now we need it for calculations
        // so we need to take into account this one token
        deposit += Uint128::one();
        let mut stake_change = calc_diff(ideal_stake, d.stake);

        if deposit < stake_change {
            stake_change = deposit;
        }

        deposit -= stake_change;

        deposit_changes.push((d.valoper_address, stake_change));
    }

    Ok(deposit_changes)
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

#[cfg(test)]
mod tests {
    use drop_staking_base::msg::distribution::Delegation;

    use super::*;

    #[test]
    fn calc_ideal_deposit_single_from_zero() {
        let stake = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::zero(),
            total_weight: 10,
            delegations: vec![Delegation {
                valoper_address: "valoper1".to_string(),
                stake: Uint128::zero(),
                weight: 10,
            }],
        };

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![("valoper1".to_string(), Uint128::from(100u128))]
        );
    }

    #[test]
    fn calc_ideal_deposit_single_with_distribution() {
        let stake = Uint128::from(50u128);

        let delegations = Delegations {
            total: Uint128::from(100u128),
            total_weight: 10,
            delegations: vec![Delegation {
                valoper_address: "valoper1".to_string(),
                stake: Uint128::from(100u128),
                weight: 10,
            }],
        };

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![("valoper1".to_string(), Uint128::from(50u128))]
        );
    }

    #[test]
    fn calc_ideal_deposit_from_zero() {
        let stake = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::zero(),
            total_weight: 70,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::zero(),
                    weight: 10,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::zero(),
                    weight: 20,
                },
                Delegation {
                    valoper_address: "valoper3".to_string(),
                    stake: Uint128::zero(),
                    weight: 40,
                },
            ],
        };

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::from(15u128)),
                ("valoper2".to_string(), Uint128::from(29u128)),
                ("valoper3".to_string(), Uint128::from(56u128))
            ]
        );
    }

    #[test]
    fn calc_ideal_deposit_with_distributions() {
        let stake = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::from(100u128),
            total_weight: 70,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(15u128),
                    weight: 10,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::from(29u128),
                    weight: 20,
                },
                Delegation {
                    valoper_address: "valoper3".to_string(),
                    stake: Uint128::from(56u128),
                    weight: 40,
                },
            ],
        };

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::from(14u128)),
                ("valoper2".to_string(), Uint128::from(29u128)),
                ("valoper3".to_string(), Uint128::from(57u128))
            ]
        );
    }

    #[test]
    fn calc_ideal_deposit_with_missed() {
        let stake = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::from(141u128),
            total_weight: 70,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(15u128),
                    weight: 10,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::from(70u128),
                    weight: 20,
                },
                Delegation {
                    valoper_address: "valoper3".to_string(),
                    stake: Uint128::from(56u128),
                    weight: 40,
                },
            ],
        };

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::from(20u128)),
                ("valoper2".to_string(), Uint128::one()),
                ("valoper3".to_string(), Uint128::from(79u128))
            ]
        );
    }

    #[test]
    fn calc_ideal_deposit_new_second_validator() {
        let stake = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::from(200u128),
            total_weight: 70,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(110u128),
                    weight: 10,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::zero(),
                    weight: 40,
                },
                Delegation {
                    valoper_address: "valoper3".to_string(),
                    stake: Uint128::from(90u128),
                    weight: 20,
                },
            ],
        };

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::one()),
                ("valoper2".to_string(), Uint128::from(98u128)),
                ("valoper3".to_string(), Uint128::one())
            ]
        );
    }

    #[test]
    fn calc_ideal_deposit_new_third_validator() {
        let stake = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::from(350u128),
            total_weight: 90,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(150u128),
                    weight: 10,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::from(200u128),
                    weight: 40,
                },
                Delegation {
                    valoper_address: "valoper3".to_string(),
                    stake: Uint128::zero(),
                    weight: 40,
                },
            ],
        };

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::one()),
                ("valoper2".to_string(), Uint128::one()),
                ("valoper3".to_string(), Uint128::from(98u128))
            ]
        );
    }

    #[test]
    fn calc_ideal_withdraw_single_from_zero() {
        let withdraw = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::zero(),
            total_weight: 10,
            delegations: vec![Delegation {
                valoper_address: "valoper1".to_string(),
                stake: Uint128::zero(),
                weight: 10,
            }],
        };

        let error = calc_withdraw(withdraw, delegations).unwrap_err();

        assert_eq!(error, ContractError::TooBigWithdraw {});
    }

    #[test]
    fn calc_ideal_withdraw_single_not_enough_stake() {
        let withdraw = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::from(50u128),
            total_weight: 10,
            delegations: vec![Delegation {
                valoper_address: "valoper1".to_string(),
                stake: Uint128::from(50u128),
                weight: 10,
            }],
        };

        let error = calc_withdraw(withdraw, delegations).unwrap_err();

        assert_eq!(error, ContractError::TooBigWithdraw {});
    }

    #[test]
    fn calc_ideal_withdraw_single_with_distribution() {
        let withdraw = Uint128::from(50u128);

        let delegations = Delegations {
            total: Uint128::from(100u128),
            total_weight: 10,
            delegations: vec![Delegation {
                valoper_address: "valoper1".to_string(),
                stake: Uint128::from(100u128),
                weight: 10,
            }],
        };

        let distribution = calc_withdraw(withdraw, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![("valoper1".to_string(), Uint128::from(50u128)),]
        );
    }

    #[test]
    fn calc_ideal_withdraw_from_enough_stake() {
        let withdraw = Uint128::from(100u128);

        let delegations = Delegations {
            total: Uint128::from(310u128),
            total_weight: 70,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(70u128),
                    weight: 10,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::from(90u128),
                    weight: 20,
                },
                Delegation {
                    valoper_address: "valoper3".to_string(),
                    stake: Uint128::from(150u128),
                    weight: 40,
                },
            ],
        };

        let distribution = calc_withdraw(withdraw, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::from(40u128)),
                ("valoper2".to_string(), Uint128::from(30u128)),
                ("valoper3".to_string(), Uint128::from(30u128))
            ]
        );
    }

    #[test]
    fn calc_ideal_withdraw_only_single_delegation() {
        let withdraw = Uint128::from(50u128);

        let delegations = Delegations {
            total: Uint128::from(750u128),
            total_weight: 70,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(100u128),
                    weight: 10,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::from(250u128),
                    weight: 20,
                },
                Delegation {
                    valoper_address: "valoper3".to_string(),
                    stake: Uint128::from(400u128),
                    weight: 40,
                },
            ],
        };

        let distribution = calc_withdraw(withdraw, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::one()),
                ("valoper2".to_string(), Uint128::from(48u128)),
                ("valoper3".to_string(), Uint128::one())
            ]
        );
    }

    #[test]
    fn calc_ideal_withdraw_from_two_delegations_too_big() {
        let withdraw = Uint128::from(1000u128);

        let delegations = Delegations {
            total: Uint128::from(1000u128),
            total_weight: 2,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(500u128),
                    weight: 1,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::from(500u128),
                    weight: 1,
                },
            ],
        };

        let error = calc_withdraw(withdraw, delegations).unwrap_err();

        assert_eq!(error, ContractError::TooBigWithdraw {});
    }

    #[test]
    fn calc_ideal_withdraw_from_two_delegations() {
        let withdraw = Uint128::from(998u128);

        let delegations = Delegations {
            total: Uint128::from(1000u128),
            total_weight: 2,
            delegations: vec![
                Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(500u128),
                    weight: 1,
                },
                Delegation {
                    valoper_address: "valoper2".to_string(),
                    stake: Uint128::from(500u128),
                    weight: 1,
                },
            ],
        };

        let distribution = calc_withdraw(withdraw, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                ("valoper1".to_string(), Uint128::from(499u128)),
                ("valoper2".to_string(), Uint128::from(499u128))
            ]
        );
    }
}
