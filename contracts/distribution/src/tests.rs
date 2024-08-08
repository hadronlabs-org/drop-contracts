use cosmwasm_std::{
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_json_binary, Empty, OwnedDeps, Querier, Uint128,
};
use drop_staking_base::msg::distribution::{Delegation, Delegations, QueryMsg};
use std::marker::PhantomData;

fn mock_dependencies<Q: Querier + Default>() -> OwnedDeps<MockStorage, MockApi, Q, Empty> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies::<MockQuerier>();
    let res = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::distribution::InstantiateMsg {},
    )
    .unwrap();

    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(cosmwasm_std::Event::new(
            "crates.io:drop-staking__drop-distribution-instantiate".to_string()
        ))
    );
}

#[test]
fn query_deposit_calculation() {
    let deps = mock_dependencies::<MockQuerier>();
    {
        let response = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CalcDeposit {
                deposit: Uint128::from(100u128),
                delegations: Delegations {
                    total: Uint128::zero(),
                    total_weight: 10,
                    delegations: vec![
                        Delegation {
                            valoper_address: "valoper1".to_string(),
                            stake: Uint128::zero(),
                            weight: 0u64,
                        },
                        Delegation {
                            valoper_address: "valoper2".to_string(),
                            stake: Uint128::zero(),
                            weight: 10u64,
                        },
                    ],
                },
            },
        )
        .unwrap();

        assert_eq!(
            response,
            to_json_binary(&vec![("valoper2".to_string(), Uint128::from(100u128))]).unwrap()
        );
    }
    {
        let response = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CalcDeposit {
                deposit: Uint128::from(100u128),
                delegations: Delegations {
                    total: Uint128::zero(),
                    total_weight: 20,
                    delegations: vec![
                        Delegation {
                            valoper_address: "valoper1".to_string(),
                            stake: Uint128::zero(),
                            weight: 10u64,
                        },
                        Delegation {
                            valoper_address: "valoper2".to_string(),
                            stake: Uint128::zero(),
                            weight: 10u64,
                        },
                    ],
                },
            },
        )
        .unwrap();

        assert_eq!(
            response,
            to_json_binary(&vec![
                ("valoper1".to_string(), Uint128::from(50u128)),
                ("valoper2".to_string(), Uint128::from(50u128))
            ])
            .unwrap()
        );
    }
}

#[test]
fn query_withdraw_calculation() {
    let deps = mock_dependencies::<MockQuerier>();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::CalcWithdraw {
            withdraw: Uint128::from(50u128),
            delegations: Delegations {
                total: Uint128::from(100u128),
                total_weight: 10,
                delegations: vec![Delegation {
                    valoper_address: "valoper1".to_string(),
                    stake: Uint128::from(100u128),
                    weight: 10u64,
                }],
            },
        },
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&vec![("valoper1".to_string(), Uint128::from(50u128))]).unwrap()
    );
}
