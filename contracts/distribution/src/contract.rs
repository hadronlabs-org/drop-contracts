use cosmwasm_std::{entry_point, to_json_binary, Attribute, Decimal, Deps, Uint128};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use lido_helpers::answer::response;
use lido_staking_base::error::distribution::{ContractError, ContractResult};
use lido_staking_base::msg::distribution::{Delegation, IdealDelegation, InstantiateMsg, QueryMsg};
use neutron_sdk::bindings::msg::NeutronMsg;

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let empty_attr: Vec<Attribute> = Vec::new();
    Ok(response("instantiate", CONTRACT_NAME, empty_attr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

fn calculate_total_stake_withdraw(
    delegations: &[Delegation],
    withdraw: Uint128,
) -> ContractResult<Uint128> {
    let total: Uint128 = delegations.iter().map(|d| d.stake).sum();
    if total < withdraw {
        return Err(ContractError::TooBigWithdraw {});
    }
    Ok(total - withdraw)
}

fn calculate_total_stake_deposit(
    delegations: &[Delegation],
    deposit: Uint128,
) -> ContractResult<Uint128> {
    let total: Uint128 = delegations.iter().map(|d| d.stake).sum();
    Ok(total + deposit)
}

fn distribute_stake_withdraw(
    withdraw: Uint128,
    ideal_distribution: Vec<IdealDelegation>,
) -> Vec<IdealDelegation> {
    let mut stake_left = withdraw;
    let mut distribution: Vec<IdealDelegation> = Vec::new();

    for d in ideal_distribution {
        if d.ideal_stake >= d.current_stake {
            continue;
        }

        let mut dist = d.clone();
        let mut stake_diff = dist.current_stake - dist.ideal_stake;

        if stake_left < stake_diff {
            stake_diff = stake_left;
        }

        stake_left -= stake_diff;
        dist.stake_change = stake_diff;
        distribution.push(dist);

        if stake_left == Uint128::zero() {
            break;
        }
    }

    distribution
}

fn distribute_stake_deposit(
    deposit: Uint128,
    ideal_distribution: Vec<IdealDelegation>,
) -> Vec<IdealDelegation> {
    let mut stake_left = deposit;
    let mut distribution: Vec<IdealDelegation> = Vec::new();

    for d in ideal_distribution {
        if d.ideal_stake <= d.current_stake {
            continue;
        }

        let mut dist = d.clone();
        let mut stake_diff = dist.ideal_stake - dist.current_stake;

        if stake_left < stake_diff {
            stake_diff = stake_left;
        }

        stake_left -= stake_diff;
        dist.stake_change = stake_diff;
        distribution.push(dist);

        if stake_left == Uint128::zero() {
            break;
        }
    }

    distribution
}

/// Calculates the ideal withdrawal of stake among the given withdraw amount.
pub fn calc_withdraw(
    withdraw: Uint128,
    delegations: Vec<Delegation>,
) -> ContractResult<Vec<IdealDelegation>> {
    let total_stake: Uint128 = calculate_total_stake_withdraw(&delegations, withdraw)?;
    let ideal_distribution = calc_ideal_stake(total_stake, delegations)?;
    let distribution = distribute_stake_withdraw(withdraw, ideal_distribution);

    Ok(distribution)
}

/// Calculates the ideal distribution of stake among the given delegations.
pub fn calc_deposit(
    deposit: Uint128,
    delegations: Vec<Delegation>,
) -> ContractResult<Vec<IdealDelegation>> {
    let total_stake: Uint128 = calculate_total_stake_deposit(&delegations, deposit)?;
    let ideal_distribution = calc_ideal_stake(total_stake, delegations)?;
    let distribution = distribute_stake_deposit(deposit, ideal_distribution);

    Ok(distribution)
}

pub fn calc_ideal_stake(
    mut total_stake: Uint128,
    delegations: Vec<Delegation>,
) -> ContractResult<Vec<IdealDelegation>> {
    let total_weight: u64 = delegations.iter().map(|d| d.weight).sum();

    let stake_per_weight = Decimal::from_ratio(total_stake, total_weight);

    let mut distribution: Vec<IdealDelegation> = Vec::new();
    for d in delegations {
        let weight = Decimal::from_atomics(d.weight, 0)?;
        let mut ideal_stake = stake_per_weight.checked_mul(weight)?.to_uint_ceil(); // ceil used to consume all available stake

        // If the ideal stake is more than the total stake, we can't increase it.
        // It means that we distributed all available stake and can't contunue after last
        // delegation.
        if total_stake < ideal_stake {
            ideal_stake = total_stake;
        }

        let ideal_delegation = IdealDelegation {
            valoper_address: d.valoper_address,
            ideal_stake,
            current_stake: d.stake,
            stake_change: Uint128::zero(),
            weight: d.weight,
        };

        distribution.push(ideal_delegation);

        total_stake -= ideal_stake;

        // If the total stake is zero, we distributed all available stake and can't continue.
        if total_stake == Uint128::zero() {
            break;
        }
    }

    Ok(distribution)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_ideal_deposit_single_from_zero() {
        let stake = Uint128::from(100u128);

        let delegations = vec![Delegation {
            valoper_address: "valoper1".to_string(),
            stake: Uint128::zero(),
            weight: 10,
        }];

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![IdealDelegation {
                valoper_address: "valoper1".to_string(),
                ideal_stake: Uint128::from(100u128),
                current_stake: Uint128::zero(),
                stake_change: Uint128::from(100u128),
                weight: 10,
            },]
        );
    }

    #[test]
    fn calc_ideal_deposit_single_with_distribution() {
        let stake = Uint128::from(50u128);

        let delegations = vec![Delegation {
            valoper_address: "valoper1".to_string(),
            stake: Uint128::from(100u128),
            weight: 10,
        }];

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![IdealDelegation {
                valoper_address: "valoper1".to_string(),
                ideal_stake: Uint128::from(150u128),
                current_stake: Uint128::from(100u128),
                stake_change: Uint128::from(50u128),
                weight: 10,
            },]
        );
    }

    #[test]
    fn calc_ideal_deposit_from_zero() {
        let stake = Uint128::from(100u128);

        let delegations = vec![
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
        ];

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                IdealDelegation {
                    valoper_address: "valoper1".to_string(),
                    ideal_stake: Uint128::from(15u128),
                    current_stake: Uint128::zero(),
                    stake_change: Uint128::from(15u128),
                    weight: 10,
                },
                IdealDelegation {
                    valoper_address: "valoper2".to_string(),
                    ideal_stake: Uint128::from(29u128),
                    current_stake: Uint128::zero(),
                    stake_change: Uint128::from(29u128),
                    weight: 20,
                },
                IdealDelegation {
                    valoper_address: "valoper3".to_string(),
                    ideal_stake: Uint128::from(56u128),
                    current_stake: Uint128::zero(),
                    stake_change: Uint128::from(56u128),
                    weight: 40,
                }
            ]
        );
    }

    #[test]
    fn calc_ideal_deposit_with_distributions() {
        let stake = Uint128::from(100u128);

        let delegations = vec![
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
        ];

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                IdealDelegation {
                    valoper_address: "valoper1".to_string(),
                    ideal_stake: Uint128::from(29u128),
                    current_stake: Uint128::from(15u128),
                    stake_change: Uint128::from(14u128),
                    weight: 10,
                },
                IdealDelegation {
                    valoper_address: "valoper2".to_string(),
                    ideal_stake: Uint128::from(58u128),
                    current_stake: Uint128::from(29u128),
                    stake_change: Uint128::from(29u128),
                    weight: 20,
                },
                IdealDelegation {
                    valoper_address: "valoper3".to_string(),
                    ideal_stake: Uint128::from(113u128),
                    current_stake: Uint128::from(56u128),
                    stake_change: Uint128::from(57u128),
                    weight: 40,
                }
            ]
        );
    }

    #[test]
    fn calc_ideal_deposit_with_missed() {
        let stake = Uint128::from(100u128);

        let delegations = vec![
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
        ];

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                IdealDelegation {
                    valoper_address: "valoper1".to_string(),
                    ideal_stake: Uint128::from(35u128),
                    current_stake: Uint128::from(15u128),
                    stake_change: Uint128::from(20u128),
                    weight: 10,
                },
                IdealDelegation {
                    valoper_address: "valoper3".to_string(),
                    ideal_stake: Uint128::from(137u128),
                    current_stake: Uint128::from(56u128),
                    stake_change: Uint128::from(80u128),
                    weight: 40,
                }
            ]
        );
    }

    #[test]
    fn calc_ideal_deposit_new_second_validator() {
        let stake = Uint128::from(100u128);

        let delegations = vec![
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
        ];

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![IdealDelegation {
                valoper_address: "valoper2".to_string(),
                ideal_stake: Uint128::from(172u128),
                current_stake: Uint128::zero(),
                stake_change: Uint128::from(100u128),
                weight: 40,
            },]
        );
    }

    #[test]
    fn calc_ideal_deposit_new_third_validator() {
        let stake = Uint128::from(100u128);

        let delegations = vec![
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
        ];

        let distribution = calc_deposit(stake, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![IdealDelegation {
                valoper_address: "valoper3".to_string(),
                ideal_stake: Uint128::from(200u128),
                current_stake: Uint128::zero(),
                stake_change: Uint128::from(100u128),
                weight: 40,
            },]
        );
    }

    #[test]
    fn calc_ideal_withdraw_single_from_zero() {
        let withdraw = Uint128::from(100u128);

        let delegations = vec![Delegation {
            valoper_address: "valoper1".to_string(),
            stake: Uint128::zero(),
            weight: 10,
        }];

        let error = calc_withdraw(withdraw, delegations).unwrap_err();

        assert_eq!(error, ContractError::TooBigWithdraw {});
    }

    #[test]
    fn calc_ideal_withdraw_single_not_enough_stake() {
        let withdraw = Uint128::from(100u128);

        let delegations = vec![Delegation {
            valoper_address: "valoper1".to_string(),
            stake: Uint128::from(50u128),
            weight: 10,
        }];

        let error = calc_withdraw(withdraw, delegations).unwrap_err();

        assert_eq!(error, ContractError::TooBigWithdraw {});
    }

    #[test]
    fn calc_ideal_withdraw_single_with_distribution() {
        let withdraw = Uint128::from(50u128);

        let delegations = vec![Delegation {
            valoper_address: "valoper1".to_string(),
            stake: Uint128::from(100u128),
            weight: 10,
        }];

        let distribution = calc_withdraw(withdraw, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![IdealDelegation {
                valoper_address: "valoper1".to_string(),
                ideal_stake: Uint128::from(50u128),
                current_stake: Uint128::from(100u128),
                stake_change: Uint128::from(50u128),
                weight: 10,
            },]
        );
    }

    #[test]
    fn calc_ideal_withdraw_from_enough_stake() {
        let withdraw = Uint128::from(100u128);

        let delegations = vec![
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
        ];

        let distribution = calc_withdraw(withdraw, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![
                IdealDelegation {
                    valoper_address: "valoper1".to_string(),
                    ideal_stake: Uint128::from(30u128),
                    current_stake: Uint128::from(70u128),
                    stake_change: Uint128::from(40u128),
                    weight: 10,
                },
                IdealDelegation {
                    valoper_address: "valoper2".to_string(),
                    ideal_stake: Uint128::from(60u128),
                    current_stake: Uint128::from(90u128),
                    stake_change: Uint128::from(30u128),
                    weight: 20,
                },
                IdealDelegation {
                    valoper_address: "valoper3".to_string(),
                    ideal_stake: Uint128::from(120u128),
                    current_stake: Uint128::from(150u128),
                    stake_change: Uint128::from(30u128),
                    weight: 40,
                },
            ]
        );
    }

    #[test]
    fn calc_ideal_withdraw_only_single_delegation() {
        let withdraw = Uint128::from(50u128);

        let delegations = vec![
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
        ];

        let distribution = calc_withdraw(withdraw, delegations).unwrap();

        assert_eq!(
            distribution,
            vec![IdealDelegation {
                valoper_address: "valoper2".to_string(),
                ideal_stake: Uint128::from(200u128),
                current_stake: Uint128::from(250u128),
                stake_change: Uint128::from(50u128),
                weight: 20,
            },]
        );
    }
}
