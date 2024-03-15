use cosmwasm_std::{to_json_binary, Addr, Binary, Coin, Delegation, Uint128};
use lido_puppeteer_base::r#trait::PuppeteerReconstruct;
use neutron_sdk::interchain_queries::v047::helpers::create_account_denom_balance_key;
use neutron_sdk::NeutronResult;
use neutron_sdk::{bindings::types::StorageValue, interchain_queries::helpers::decode_and_convert};
use prost::Message;

use super::puppeteer::{BalancesAndDelegations, MultiBalances};

#[test]
fn test_reconstruct_multi_balances() {
    let coin1 = cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
        denom: "uatom".to_string(),
        amount: "1000".to_string(),
    };
    let coin2 = cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
        denom: "utia".to_string(),
        amount: "2000".to_string(),
    };
    let mut buf_coin1 = Vec::new();
    coin1.encode(&mut buf_coin1).unwrap();
    let mut buf_coin2 = Vec::new();
    coin2.encode(&mut buf_coin2).unwrap();
    let key_atom = create_account_denom_balance_key(
        decode_and_convert("cosmos1hdga6p84cpc6gulk9ruxy5w0vpfx9dv83ku59r").unwrap(),
        "uatom",
    )
    .unwrap();
    let key_tia = create_account_denom_balance_key(
        decode_and_convert("cosmos1hdga6p84cpc6gulk9ruxy5w0vpfx9dv83ku59r").unwrap(),
        "utia",
    )
    .unwrap();
    let storage_values: Vec<StorageValue> = vec![
        StorageValue {
            storage_prefix: "prefix".to_string(),
            key: Binary::from(key_atom),
            value: buf_coin1.into(),
        },
        StorageValue {
            storage_prefix: "prefix".to_string(),
            key: Binary::from(key_tia),
            value: buf_coin2.into(),
        },
    ];
    let result = MultiBalances::reconstruct(&storage_values, "0.0.1").unwrap();
    let expected_coins = vec![
        cosmwasm_std::Coin {
            denom: "uatom".to_string(),
            amount: Uint128::from(1000u128),
        },
        cosmwasm_std::Coin {
            denom: "utia".to_string(),
            amount: Uint128::from(2000u128),
        },
    ];
    assert_eq!(result.coins, expected_coins);
}

#[test]
fn test_reconstruct_balance_and_delegations_no_delegations() {
    let coin = cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
        denom: "uatom".to_string(),
        amount: "1000".to_string(),
    };
    let mut buf_coin = Vec::new();
    coin.encode(&mut buf_coin).unwrap();
    let key = create_account_denom_balance_key(
        decode_and_convert("cosmos1hdga6p84cpc6gulk9ruxy5w0vpfx9dv83ku59r").unwrap(),
        "uatom",
    )
    .unwrap();
    let storage_values: Vec<StorageValue> = vec![
        StorageValue {
            storage_prefix: "".to_string(),
            key: Binary::from(key),
            value: buf_coin.into(),
        },
        StorageValue {
            storage_prefix: "".to_string(),
            key: Binary::from("denom".as_bytes()),
            value: to_json_binary(&"uatom".to_string()).unwrap(),
        },
    ];
    let result: NeutronResult<BalancesAndDelegations> =
        BalancesAndDelegations::reconstruct(&storage_values, "0.0.1");
    match result {
        Ok(balances_and_delegations) => {
            let expected_coins = vec![cosmwasm_std::Coin {
                denom: "uatom".to_string(),
                amount: Uint128::from(1000u128),
            }];
            assert_eq!(balances_and_delegations.balances.coins, expected_coins);

            let expected_delegations: Vec<Delegation> = vec![];
            assert_eq!(
                balances_and_delegations.delegations.delegations,
                expected_delegations
            );
        }
        Err(e) => {
            panic!("reconstruct method returned an error: {:?}", e);
        }
    }
}

#[test]
fn test_reconstruct_balance_and_delegations_with_delegations() {
    let coin = cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
        denom: "uatom".to_string(),
        amount: "1000".to_string(),
    };
    let mut buf_coin = Vec::new();
    coin.encode(&mut buf_coin).unwrap();
    let key = create_account_denom_balance_key(
        decode_and_convert("cosmos1hdga6p84cpc6gulk9ruxy5w0vpfx9dv83ku59r").unwrap(),
        "uatom",
    )
    .unwrap();
    let mut storage_values: Vec<StorageValue> = vec![
        StorageValue {
            storage_prefix: "prefix".to_string(),
            key: Binary::from(key),
            value: buf_coin.into(),
        },
        StorageValue {
            storage_prefix: "prefix".to_string(),
            key: Binary::from("denom".as_bytes()),
            value: to_json_binary(&"uatom".to_string()).unwrap(),
        },
    ];

    let delegation = cosmos_sdk_proto::cosmos::staking::v1beta1::Delegation {
        delegator_address: "delegator".to_string(),
        validator_address: "validator".to_string(),
        shares: "1000".to_string(),
    };
    let mut buf = Vec::new();
    delegation.encode(&mut buf).unwrap();
    storage_values.push(StorageValue {
        storage_prefix: "prefix".to_string(),
        key: Binary::from("delegation".as_bytes()),
        value: buf.into(),
    });

    let validator = cosmos_sdk_proto::cosmos::staking::v1beta1::Validator {
        operator_address: "operator".to_string(),
        consensus_pubkey: None,
        jailed: false,
        status: 1,
        tokens: "1000".to_string(),
        delegator_shares: "1000".to_string(),
        description: None,
        unbonding_height: 0,
        unbonding_time: None,
        commission: None,
        min_self_delegation: "1000".to_string(),
    };
    let mut buf = Vec::new();
    validator.encode(&mut buf).unwrap();
    storage_values.push(StorageValue {
        storage_prefix: "prefix".to_string(),
        key: Binary::from("validator".as_bytes()),
        value: buf.into(),
    });

    let result: NeutronResult<BalancesAndDelegations> =
        PuppeteerReconstruct::reconstruct(&storage_values, "0.0.1");
    match result {
        Ok(balances_and_delegations) => {
            let expected_coins = vec![cosmwasm_std::Coin {
                denom: "uatom".to_string(),
                amount: Uint128::from(1000u128),
            }];
            assert_eq!(balances_and_delegations.balances.coins, expected_coins);

            let expected_delegations: Vec<Delegation> = vec![Delegation {
                delegator: Addr::unchecked("delegator"),
                validator: "validator".to_string(),
                amount: Coin {
                    denom: "uatom".to_string(),
                    amount: Uint128::from(1000u128),
                },
            }];
            assert_eq!(
                balances_and_delegations.delegations.delegations,
                expected_delegations
            );
        }
        Err(e) => {
            panic!("reconstruct method returned an error: {:?}", e);
        }
    }
}
