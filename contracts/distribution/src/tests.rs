use super::contract::{calc_deposit, calc_withdraw};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_json_binary, Empty, OwnedDeps, Querier, Uint128,
};
use drop_staking_base::{
    error::distribution::ContractError,
    msg::distribution::{Delegation, QueryMsg},
};
use std::marker::PhantomData;

fn mock_dependencies<Q: Querier + Default>() -> OwnedDeps<MockStorage, MockApi, Q, Empty> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    }
}

fn make_delegations(delegations: &[(&str, u128, u128, u64)]) -> Vec<Delegation> {
    delegations
        .iter()
        .map(|d| Delegation {
            valoper_address: d.0.to_string(),
            stake: Uint128::new(d.1),
            on_top: Uint128::new(d.2),
            weight: d.3,
        })
        .collect()
}

fn assert_distributions_eq(mut left: Vec<(String, Uint128)>, right: &[(&str, u128)]) {
    let mut right = right
        .iter()
        .map(|(a, b)| (a.to_string(), Uint128::new(*b)))
        .collect::<Vec<_>>();
    left.sort();
    right.sort();
    assert_eq!(left, right);
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::distribution::InstantiateMsg {},
    )
    .unwrap();

    assert_eq!(
        response,
        cosmwasm_std::Response::new().add_event(cosmwasm_std::Event::new(
            "crates.io:drop-staking__drop-distribution-instantiate".to_string()
        ))
    );
}

#[test]
fn query_deposit_calculation() {
    let deps = mock_dependencies::<MockQuerier>();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::CalcDeposit {
            deposit: Uint128::new(100),
            delegations: make_delegations(&[("1", 0, 0, 0), ("2", 0, 0, 10)]),
        },
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&vec![("2".to_string(), Uint128::new(100))]).unwrap()
    );
}

#[test]
fn query_withdraw_calculation() {
    let deps = mock_dependencies::<MockQuerier>();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::CalcWithdraw {
            withdraw: Uint128::new(50),
            delegations: make_delegations(&[("1", 100, 0, 10)]),
        },
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&vec![("1".to_string(), Uint128::new(50))]).unwrap()
    );
}

#[test]
fn calc_deposit_single_from_zero() {
    let stake = Uint128::new(100);
    let delegations = make_delegations(&[("1", 0, 0, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 100)]);
}

#[test]
fn calc_deposit_single_with_distribution() {
    let stake = Uint128::new(50);
    let delegations = make_delegations(&[("1", 100, 0, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 50)]);
}

#[test]
fn calc_deposit_from_zero() {
    let stake = Uint128::new(100);
    let delegations = make_delegations(&[("1", 0, 0, 10), ("2", 0, 0, 20), ("3", 0, 0, 40)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 15), ("2", 29), ("3", 56)]);
}

#[test]
fn calc_deposit_with_distributions() {
    let stake = Uint128::new(100);
    let delegations = make_delegations(&[("1", 15, 0, 10), ("2", 29, 0, 20), ("3", 56, 0, 40)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 14), ("2", 29), ("3", 57)]);
}

#[test]
fn calc_deposit_with_missed() {
    let stake = Uint128::new(100);
    let delegations = make_delegations(&[("1", 15, 0, 10), ("2", 70, 0, 20), ("3", 56, 0, 40)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 20), ("3", 80)]);
}

#[test]
fn calc_deposit_new_second_validator() {
    let stake = Uint128::new(100);
    let delegations = make_delegations(&[("1", 110, 0, 10), ("2", 0, 0, 40)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 100)]);
}

#[test]
fn calc_deposit_new_third_validator() {
    let stake = Uint128::new(100);
    let delegations = make_delegations(&[("1", 150, 0, 10), ("2", 200, 0, 40), ("3", 0, 0, 40)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("3", 100)]);
}

#[test]
fn calc_withdraw_single_from_zero() {
    let withdraw = Uint128::new(100);
    let delegations = make_delegations(&[("1", 0, 0, 10)]);

    let error = calc_withdraw(withdraw, delegations).unwrap_err();
    assert_eq!(error, ContractError::TooBigWithdraw {});
}

#[test]
fn calc_withdraw_single_not_enough_stake() {
    let withdraw = Uint128::new(100);
    let delegations = make_delegations(&[("1", 50, 0, 10)]);

    let error = calc_withdraw(withdraw, delegations).unwrap_err();
    assert_eq!(error, ContractError::TooBigWithdraw {});
}

#[test]
fn calc_withdraw_single_with_distribution() {
    let withdraw = Uint128::new(50);
    let delegations = make_delegations(&[("1", 100, 0, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 50)]);
}

#[test]
fn calc_withdraw_from_enough_stake() {
    let withdraw = Uint128::new(100);
    let delegations = make_delegations(&[("1", 70, 0, 10), ("2", 90, 0, 20), ("3", 150, 0, 40)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 40), ("2", 30), ("3", 30)]);
}

#[test]
fn calc_withdraw_only_single_delegation() {
    let withdraw = Uint128::new(50);
    let delegations = make_delegations(&[("1", 100, 0, 10), ("2", 250, 0, 20), ("3", 400, 0, 40)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 50)]);
}

#[test]
fn calc_withdraw_from_two_delegations_too_big() {
    let withdraw = Uint128::new(1001);
    let delegations = make_delegations(&[("1", 500, 0, 1), ("2", 500, 0, 1)]);

    let error = calc_withdraw(withdraw, delegations).unwrap_err();
    assert_eq!(error, ContractError::TooBigWithdraw {});
}

#[test]
fn calc_withdraw_from_two_delegations() {
    let withdraw = Uint128::new(998);
    let delegations = make_delegations(&[("1", 500, 0, 1), ("2", 500, 0, 1)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 499), ("2", 499)]);
}

#[test]
fn on_top_deposit_one_of_one_satisfy_exactly() {
    let stake = Uint128::new(15);
    let delegations = make_delegations(&[("1", 15, 30, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 15)]);
}

#[test]
fn on_top_deposit_one_of_one_satisfy_over() {
    let stake = Uint128::new(20);
    let delegations = make_delegations(&[("1", 15, 30, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 20)]);
}

#[test]
fn on_top_deposit_one_of_two_satisfy_exactly() {
    let stake = Uint128::new(10);
    let delegations = make_delegations(&[("1", 20, 10, 10), ("2", 0, 10, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 10)]);
}

#[test]
fn on_top_deposit_two_of_two_satisfy_exactly() {
    let stake = Uint128::new(20);
    let delegations = make_delegations(&[("1", 20, 30, 10), ("2", 0, 10, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 10), ("2", 10)]);
}

#[test]
fn on_top_deposit_two_of_two_satisfy_over() {
    let stake = Uint128::new(50);
    let delegations = make_delegations(&[("1", 20, 20, 10), ("2", 10, 30, 20)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 10), ("2", 40)]);
}

#[test]
fn on_top_deposit_two_of_three() {
    let stake = Uint128::new(80);
    let delegations = make_delegations(&[("1", 40, 20, 10), ("2", 0, 0, 10), ("3", 10, 30, 20)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 20), ("3", 60)]);
}

#[test]
fn on_top_deposit_one_of_one_undersatisfy() {
    let stake = Uint128::new(20);
    let delegations = make_delegations(&[("1", 10, 50, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 20)]);
}

#[test]
fn on_top_deposit_two_of_two_undersatisfy() {
    let stake = Uint128::new(40);
    let delegations = make_delegations(&[("1", 10, 40, 10), ("2", 10, 80, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 10), ("2", 30)]);
}

#[test]
fn on_top_deposit_one_of_three_undersatisfy() {
    let stake = Uint128::new(10);
    let delegations = make_delegations(&[("1", 40, 20, 10), ("2", 10, 40, 10), ("3", 20, 40, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 10)]);
}

#[test]
fn on_top_deposit_one_of_three_oversatisfacty() {
    let stake = Uint128::new(20);
    let delegations = make_delegations(&[("1", 100, 0, 10), ("2", 100, 20, 10), ("3", 100, 0, 10)]);

    let distribution = calc_deposit(stake, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 20)]);
}

#[test]
fn on_top_withdraw_from_one_of_one_enough_excess() {
    let withdraw = Uint128::new(40);
    let delegations = make_delegations(&[("1", 100, 20, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 40)]);
}

#[test]
fn on_top_withdraw_from_one_of_two_satisfy_over() {
    let withdraw = Uint128::new(40);
    let delegations = make_delegations(&[("1", 100, 20, 10), ("2", 200, 20, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 40)]);
}

#[test]
fn on_top_withdraw_from_one_of_two_satisfy_exactly() {
    let withdraw = Uint128::new(40);
    let delegations = make_delegations(&[("1", 100, 60, 10), ("2", 200, 200, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 40)]);
}

#[test]
fn on_top_withdraw_from_one_of_two_mixed() {
    let withdraw = Uint128::new(10);
    let delegations = make_delegations(&[("1", 40, 40, 10), ("2", 10, 100, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 10)]);
}

#[test]
fn on_top_withdraw_from_two_of_two_satisfy_exactly() {
    let withdraw = Uint128::new(40);
    let delegations = make_delegations(&[("1", 100, 80, 10), ("2", 200, 180, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 20), ("2", 20)]);
}

#[test]
fn on_top_withdraw_from_two_of_two_undersatisfy_unbalanced() {
    let withdraw = Uint128::new(60);
    let delegations = make_delegations(&[("1", 100, 80, 10), ("2", 200, 180, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 26), ("2", 34)]);
}

#[test]
fn on_top_withdraw_from_two_of_two_undersatisfy_balanced() {
    let withdraw = Uint128::new(60);
    let delegations = make_delegations(&[("1", 100, 80, 10), ("2", 100, 80, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 30), ("2", 30)]);
}

#[test]
fn on_top_withdraw_from_one_of_three_oversatisfy() {
    let withdraw = Uint128::new(40);
    let delegations =
        make_delegations(&[("1", 120, 20, 10), ("2", 140, 40, 10), ("3", 260, 60, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("3", 40)]);
}

#[test]
fn on_top_withdraw_from_one_of_three_satisfy_exactly() {
    let withdraw = Uint128::new(40);
    let delegations = make_delegations(&[("1", 20, 20, 10), ("2", 40, 40, 10), ("3", 100, 60, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("3", 40)]);
}

#[test]
fn on_top_withdraw_from_two_of_three_satisfy_exactly() {
    let withdraw = Uint128::new(40);
    let delegations = make_delegations(&[("1", 20, 20, 10), ("2", 60, 40, 10), ("3", 80, 60, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 20), ("3", 20)]);
}

#[test]
fn on_top_withdraw_from_three_of_three_undersatisfy() {
    let withdraw = Uint128::new(30);
    let delegations = make_delegations(&[("1", 30, 20, 30), ("2", 40, 30, 70), ("3", 50, 50, 200)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 12), ("2", 13), ("3", 5)]);
}

#[test]
fn withdraw_from_one_of_two_mixed_on_top_and_normal() {
    let withdraw = Uint128::new(40);
    let delegations = make_delegations(&[("1", 200, 210, 10), ("2", 100, 0, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("2", 40)]);
}

#[test]
fn withdraw_from_two_of_two_mixed_on_top_and_normal() {
    let withdraw = Uint128::new(140);
    let delegations = make_delegations(&[("1", 200, 210, 10), ("2", 100, 0, 10)]);

    let distribution = calc_withdraw(withdraw, delegations).unwrap();
    assert_distributions_eq(distribution, &[("1", 40), ("2", 100)]);
}

#[test]
fn test_migrate_wrong_contract() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let deps_mut = deps.as_mut();

    cw2::set_contract_version(deps_mut.storage, "wrong_contract_name", "0.0.1").unwrap();

    let res = crate::contract::migrate(
        deps.as_mut(),
        mock_env(),
        drop_staking_base::msg::distribution::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}
