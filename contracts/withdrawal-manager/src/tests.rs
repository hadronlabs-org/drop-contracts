use crate::contract::execute;
use crate::error::ContractError;
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Event, Response, SubMsg, Uint128, WasmMsg,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::core::{UnbondBatch, UnbondBatchStatus, UnbondBatchStatusTimestamps};
use drop_staking_base::{
    msg::withdrawal_manager::ExecuteMsg,
    state::withdrawal_manager::{Config, CONFIG},
};

fn get_default_config() -> Config {
    Config {
        core_contract: Addr::unchecked("core_contract"),
        withdrawal_token_contract: Addr::unchecked("withdrawal_token_contract"),
        withdrawal_voucher_contract: Addr::unchecked("withdrawal_voucher_contract"),
        base_denom: "base_denom".to_string(),
    }
}

fn get_default_unbond_batch_status_timestamps() -> UnbondBatchStatusTimestamps {
    UnbondBatchStatusTimestamps {
        new: 0,
        unbond_requested: None,
        unbond_failed: None,
        unbonding: None,
        withdrawing: None,
        withdrawn: None,
        withdrawing_emergency: None,
        withdrawn_emergency: None,
    }
}

#[test]
fn test_receive_withdrawal_denoms_happy_path() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    deps.querier.add_wasm_query_response("core_contract", |_| {
        to_json_binary(&UnbondBatch {
            total_dasset_amount_to_withdraw: Uint128::from(1001u128),
            expected_native_asset_amount: Uint128::from(1001u128),
            total_unbond_items: 1,
            status: UnbondBatchStatus::Withdrawn,
            expected_release_time: 9000,
            slashing_effect: None,
            unbonded_amount: Option::from(Uint128::new(1000u128)),
            withdrawn_amount: None,
            status_timestamps: get_default_unbond_batch_status_timestamps(),
        })
        .unwrap()
    });

    let res = execute(
        deps.as_mut(),
        mock_env().clone(),
        mock_info(
            "any sender",
            &[Coin::new(
                1000,
                "factory/withdrawal_token_contract/dATOM:unbond:0",
            )],
        ),
        ExecuteMsg::ReceiveWithdrawalDenoms {},
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_submessages([SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "withdrawal_token_contract".to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::withdrawal_token::ExecuteMsg::Burn {
                        batch_id: Uint128::zero(),
                    },
                ).unwrap(),
                funds: vec![Coin::new(1000, "factory/withdrawal_token_contract/dATOM:unbond:0")],
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "any sender".to_string(),
                amount: vec![Coin {
                    denom: "base_denom".to_string(),
                    amount: Uint128::from(999u128),
                }],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "core_contract".to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::core::ExecuteMsg::UpdateWithdrawnAmount {
                        batch_id: 0u128,
                        withdrawn_amount: Uint128::from(999u128),
                    },
                ).unwrap(),
                funds: vec![],
            }))])
            .add_event(
                Event::new("crates.io:drop-staking__drop-withdrawal-manager-execute-receive_withdrawal_denoms").add_attributes(
                    vec![
                        ("action", "receive_withdrawal_denoms"),
                        ("batch_id", "0"),
                        ("payout_amount", "999"),
                        ("to_address", "any sender")
                    ]
                )
            )
    );
}

#[test]
fn test_receive_withdrawal_denoms_has_few_parts() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env().clone(),
        mock_info("any sender", &[Coin::new(1000, "factory/dATOM:unbond:0")]),
        ExecuteMsg::ReceiveWithdrawalDenoms {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::InvalidDenom {}));
}

#[test]
fn test_receive_withdrawal_denoms_has_incorrect_prefix() {
    let mut deps = mock_dependencies(&[]);
    let denom = "invalid/withdrawal_token_contract/dATOM:unbond:0";

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env().clone(),
        mock_info("any sender", &[Coin::new(1000, denom)]),
        ExecuteMsg::ReceiveWithdrawalDenoms {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::InvalidDenom {}));
}

#[test]
fn test_receive_withdrawal_denoms_has_incorrect_owner() {
    let mut deps = mock_dependencies(&[]);
    let denom = "factory/invalid/dATOM:unbond:0";

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env().clone(),
        mock_info("any sender", &[Coin::new(1000, denom)]),
        ExecuteMsg::ReceiveWithdrawalDenoms {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::InvalidDenom {}));
}

#[test]
fn test_receive_withdrawal_denoms_has_incorrect_subdenom() {
    let mut deps = mock_dependencies(&[]);
    let denom = "factory/withdrawal_token_contract/invalid";

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env().clone(),
        mock_info("any sender", &[Coin::new(1000, denom)]),
        ExecuteMsg::ReceiveWithdrawalDenoms {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::InvalidDenom {}));
}

#[test]
fn test_receive_withdrawal_denoms_has_incorrect_subdenom_batch_id() {
    let mut deps = mock_dependencies(&[]);
    let denom = "factory/withdrawal_token_contract/dATOM:unbond:invalid";

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env().clone(),
        mock_info("any sender", &[Coin::new(1000, denom)]),
        ExecuteMsg::ReceiveWithdrawalDenoms {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::InvalidDenom {}));
}

#[test]
fn test_receive_withdrawal_denoms_batch_not_withdrawn() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    deps.querier.add_wasm_query_response("core_contract", |_| {
        to_json_binary(&UnbondBatch {
            total_dasset_amount_to_withdraw: Uint128::from(1001u128),
            expected_native_asset_amount: Uint128::from(1001u128),
            total_unbond_items: 1,
            status: UnbondBatchStatus::Unbonding,
            expected_release_time: 9000,
            slashing_effect: None,
            unbonded_amount: Option::from(Uint128::new(1000u128)),
            withdrawn_amount: None,
            status_timestamps: get_default_unbond_batch_status_timestamps(),
        })
        .unwrap()
    });

    let res = execute(
        deps.as_mut(),
        mock_env().clone(),
        mock_info(
            "any sender",
            &[Coin::new(
                1000,
                "factory/withdrawal_token_contract/dATOM:unbond:0",
            )],
        ),
        ExecuteMsg::ReceiveWithdrawalDenoms {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::BatchIsNotWithdrawn {}));
}
