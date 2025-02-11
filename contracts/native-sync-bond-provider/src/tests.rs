use std::collections::HashMap;

use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BalanceResponse, Coin, CosmosMsg, Decimal, Event, Response, Uint128,
    WasmMsg,
};
use cw_ownable::{Action, Ownership};
use cw_utils::PaymentError;
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::{
    msg::native_sync_bond_provider::ConfigOptional,
    state::native_sync_bond_provider::{Config, CONFIG},
};

use crate::contract::LOCAL_DENOM;

fn get_default_config() -> Config {
    Config {
        factory_contract: Addr::unchecked("factory_contract"),
    }
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::native_sync_bond_provider::InstantiateMsg {
            owner: "owner".to_string(),
            factory_contract: "factory_contract".to_string(),
        },
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();

    assert_eq!(config, get_default_config());

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-native-sync-bond-provider-instantiate")
                .add_attributes([attr("factory_contract", "factory_contract"),])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(response, to_json_binary(&get_default_config()).unwrap());
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    CONFIG
        .save(deps_mut.storage, &get_default_config())
        .unwrap();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                factory_contract: Some("factory_contract".to_string()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn update_config_ok() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                factory_contract: Some("factory_contract_1".to_string()),
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-native-sync-bond-provider-update_config")
                .add_attributes([attr("factory_contract", "factory_contract_1"),])
        ]
    );
    assert!(response.attributes.is_empty());

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        config,
        to_json_binary(&Config {
            factory_contract: Addr::unchecked("factory_contract_1"),
        })
        .unwrap()
    );
}

#[test]
fn query_ownership() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    )
    .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::Ownership {},
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&Ownership {
            owner: Some(Addr::unchecked("core")),
            pending_owner: None,
            pending_expiry: None
        })
        .unwrap()
    );
}

#[test]
fn query_can_bond_ok() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let can_bond = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::CanBond {
            denom: LOCAL_DENOM.to_string(),
        },
    )
    .unwrap();

    assert_eq!(can_bond, to_json_binary(&true).unwrap());
}

#[test]
fn query_can_bond_false() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let can_bond = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::CanBond {
            denom: "wrong_denom".to_string(),
        },
    )
    .unwrap();

    assert_eq!(can_bond, to_json_binary(&false).unwrap());
}

#[test]
fn query_can_process_on_idle_false_if_no_funds_to_process() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    deps.querier.add_bank_query_response(
        "cosmos2contract".to_string(),
        BalanceResponse {
            amount: Coin::new(0u128, LOCAL_DENOM.to_string()),
        },
    );

    let error = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::NotEnoughToProcessIdle {
            min_stake_amount: Uint128::new(1),
            non_staked_balance: Uint128::from(0u128),
            min_ibc_transfer: Uint128::new(0),
            pending_coins: Uint128::new(0),
        }
    );
}

#[test]
fn query_can_process_on_idle_enough_non_staked_balance() {
    let mut deps = mock_dependencies(&[Coin::new(1000u128, LOCAL_DENOM)]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let res = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap();

    deps.querier.add_bank_query_response(
        "cosmos2contract".to_string(),
        BalanceResponse {
            amount: Coin::new(0u128, LOCAL_DENOM.to_string()),
        },
    );

    let res: bool = from_json(res).unwrap();

    assert!(res);
}

#[test]
fn query_can_process_on_idle_enough_contract_balance() {
    let mut deps = mock_dependencies(&[Coin::new(1000u128, LOCAL_DENOM)]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let res = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap();

    deps.querier.add_bank_query_response(
        "cosmos2contract".to_string(),
        BalanceResponse {
            amount: Coin::new(100u128, LOCAL_DENOM.to_string()),
        },
    );

    let res: bool = from_json(res).unwrap();

    assert!(res);
}

#[test]
fn query_token_amount() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: LOCAL_DENOM.to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::one(),
        },
    )
    .unwrap();

    assert_eq!(token_amount, to_json_binary(&100u128).unwrap());
}

#[test]
fn query_token_amount_half() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: LOCAL_DENOM.to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::from_atomics(Uint128::from(5u64), 1).unwrap(),
        },
    )
    .unwrap();

    assert_eq!(token_amount, to_json_binary(&200u128).unwrap());
}

#[test]
fn query_token_amount_above_one() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: LOCAL_DENOM.to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::from_atomics(Uint128::from(11u64), 1).unwrap(),
        },
    )
    .unwrap();

    assert_eq!(token_amount, to_json_binary(&90u128).unwrap());
}

#[test]
fn query_token_amount_wrong_denom() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let error = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: "wrong_denom".to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::one(),
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::InvalidDenom {}
    );
}

#[test]
fn update_ownership() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::UpdateOwnership(
            Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: None,
            },
        ),
    )
    .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_sync_bond_provider::QueryMsg::Ownership {},
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&Ownership {
            owner: Some(Addr::unchecked("core")),
            pending_owner: Some(Addr::unchecked("new_owner")),
            pending_expiry: None
        })
        .unwrap()
    );
}

#[test]
fn process_on_idle_not_core_contract() {
    let mut deps = mock_dependencies(&[]);

    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
            to_json_binary(&HashMap::from([
                ("strategy_contract", "strategy_contract"),
                ("core_contract", "core_contract"),
                ("puppeteer_contract", "puppeteer_contract"),
            ]))
            .into()
        });

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_core_contract", &[]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::Unauthorized {}
    );
}

#[test]
fn process_on_idle_delegation() {
    let mut deps = mock_dependencies(&[Coin::new(1000u128, LOCAL_DENOM)]);

    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
            to_json_binary(&HashMap::from([
                ("strategy_contract", "strategy_contract"),
                ("core_contract", "core_contract"),
                ("puppeteer_contract", "puppeteer_contract"),
            ]))
            .into()
        });

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    deps.querier
        .add_wasm_query_response("strategy_contract", |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&vec![(
                    "valoper_address".to_string(),
                    Uint128::from(1000u128),
                )])
                .unwrap(),
            )
        });

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core_contract", &[]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![attr("action", "process_on_idle"),])
            .add_event(Event::new(
                "crates.io:drop-staking__drop-native-sync-bond-provider-process_on_idle"
            ))
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
                    items: vec![("valoper_address".to_string(), Uint128::from(1000u128))],
                    reply_to: "cosmos2contract".to_string()
                })
                .unwrap(),
                funds: vec![Coin::new(1000u128, LOCAL_DENOM)],
            }))
    );
}

#[test]
fn process_on_idle_not_allowed_if_no_funds() {
    let mut deps = mock_dependencies(&[]);

    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
            to_json_binary(&HashMap::from([
                ("strategy_contract", "strategy_contract"),
                ("core_contract", "core_contract"),
                ("puppeteer_contract", "puppeteer_contract"),
            ]))
            .into()
        });

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    deps.querier.add_bank_query_response(
        "cosmos2contract".to_string(),
        BalanceResponse {
            amount: Coin::new(0u128, LOCAL_DENOM.to_string()),
        },
    );

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core_contract", &[]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::NotEnoughToProcessIdle {
            min_stake_amount: Uint128::new(1),
            non_staked_balance: Uint128::zero(),
            min_ibc_transfer: Uint128::new(0),
            pending_coins: Uint128::new(0),
        }
    );
}

#[test]
fn execute_bond() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    CONFIG
        .save(deps_mut.storage, &get_default_config())
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[Coin::new(100u128, LOCAL_DENOM)]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-native-sync-bond-provider-bond")
                .add_attributes(vec![("received_funds", "100untrn")])
        )
    );
}

#[test]
fn execute_bond_wrong_denom() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[Coin::new(100u128, "wrong_denom")]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::InvalidDenom {}
    );
}

#[test]
fn execute_bond_no_funds() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::PaymentError(
            PaymentError::NoFunds {}
        )
    );
}

#[test]
fn execute_bond_multiple_denoms() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "core",
            &[
                Coin::new(100u128, LOCAL_DENOM),
                Coin::new(100u128, "second_denom"),
            ],
        ),
        drop_staking_base::msg::native_sync_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::PaymentError(
            PaymentError::MultipleDenoms {}
        )
    );
}
