use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Event, Response, SubMsg, Uint128,
};
use cw_ownable::{Action, Ownership};
use cw_utils::PaymentError;
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::lsm_share_bond_provider::ConfigOptional;

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::lsm_share_bond_provider::InstantiateMsg {
            owner: "owner".to_string(),
            puppeteer_contract: "puppeteer_contract".to_string(),
            core_contract: "core_contract".to_string(),
            validators_set_contract: "validators_set_contract".to_string(),
            transfer_channel_id: "transfer_channel_id".to_string(),
            lsm_redeem_threshold: 100u64,
            lsm_redeem_maximum_interval: 200u64,
        },
    )
    .unwrap();

    let config = drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .load(deps.as_ref().storage)
        .unwrap();

    assert_eq!(
        config,
        drop_staking_base::state::lsm_share_bond_provider::Config {
            puppeteer_contract: Addr::unchecked("puppeteer_contract"),
            core_contract: Addr::unchecked("core_contract"),
            validators_set_contract: Addr::unchecked("validators_set_contract"),
            transfer_channel_id: "transfer_channel_id".to_string(),
            lsm_redeem_threshold: 100u64,
            lsm_redeem_maximum_interval: 200u64,
        }
    );

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-lsm-share-bond-provider-instantiate")
                .add_attributes([
                    ("puppeteer_contract", "puppeteer_contract"),
                    ("core_contract", "core_contract"),
                    ("validators_set_contract", "validators_set_contract"),
                    ("transfer_channel_id", "transfer_channel_id"),
                    ("lsm_redeem_threshold", "100"),
                    ("lsm_redeem_maximum_interval", "200")
                ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        response,
        to_json_binary(&drop_staking_base::state::lsm_share_bond_provider::Config {
            puppeteer_contract: Addr::unchecked("puppeteer_contract"),
            core_contract: Addr::unchecked("core_contract"),
            validators_set_contract: Addr::unchecked("validators_set_contract"),
            transfer_channel_id: "transfer_channel_id".to_string(),
            lsm_redeem_threshold: 100u64,
            lsm_redeem_maximum_interval: 200u64,
        })
        .unwrap()
    );
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                puppeteer_contract: Some(Addr::unchecked("puppeteer_contract_1")),
                core_contract: Some(Addr::unchecked("core_contract_1")),
                validators_set_contract: Some(Addr::unchecked("validators_set_contract_1")),
                transfer_channel_id: Some("transfer_channel_id_1".to_string()),
                lsm_redeem_threshold: Some(300u64),
                lsm_redeem_maximum_interval: Some(400u64),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::lsm_share_bond_provider::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::not_found("type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]")
        ))
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

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                puppeteer_contract: Some(Addr::unchecked("puppeteer_contract_1")),
                core_contract: Some(Addr::unchecked("core_contract_1")),
                validators_set_contract: Some(Addr::unchecked("validators_set_contract_1")),
                transfer_channel_id: Some("transfer_channel_id_1".to_string()),
                lsm_redeem_threshold: Some(300u64),
                lsm_redeem_maximum_interval: Some(400u64),
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-lsm-share-bond-provider-update_config")
                .add_attributes([
                    ("puppeteer_contract", "puppeteer_contract_1"),
                    ("core_contract", "core_contract_1"),
                    ("validators_set_contract", "validators_set_contract_1"),
                    ("transfer_channel_id", "transfer_channel_id_1"),
                    ("lsm_redeem_threshold", "300"),
                    ("lsm_redeem_maximum_interval", "400")
                ])
        ]
    );
    assert!(response.attributes.is_empty());

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::Config {},
    )
    .unwrap();

    assert_eq!(
        config,
        to_json_binary(&drop_staking_base::state::lsm_share_bond_provider::Config {
            puppeteer_contract: Addr::unchecked("puppeteer_contract_1"),
            core_contract: Addr::unchecked("core_contract_1"),
            validators_set_contract: Addr::unchecked("validators_set_contract_1"),
            transfer_channel_id: "transfer_channel_id_1".to_string(),
            lsm_redeem_threshold: 300u64,
            lsm_redeem_maximum_interval: 400u64,
        })
        .unwrap()
    );
}

#[test]
fn query_can_process_idle() {
    let deps = mock_dependencies(&[]);

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap();
    assert_eq!(response, to_json_binary(&false).unwrap());
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
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::Ownership {},
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

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let can_bond = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanBond {
            denom: "base_denom".to_string(),
        },
    )
    .unwrap();

    assert_eq!(can_bond, to_json_binary(&true).unwrap());
}

#[test]
fn query_can_bond_false() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let can_bond = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanBond {
            denom: "wrong_denom".to_string(),
        },
    )
    .unwrap();

    assert_eq!(can_bond, to_json_binary(&false).unwrap());
}

#[test]
fn query_token_amount() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TokenAmount {
            coin: Coin {
                denom: "base_denom".to_string(),
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

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TokenAmount {
            coin: Coin {
                denom: "base_denom".to_string(),
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

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TokenAmount {
            coin: Coin {
                denom: "base_denom".to_string(),
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

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let error = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TokenAmount {
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
        drop_staking_base::error::lsm_share_bond_provider::ContractError::InvalidDenom {}
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
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::UpdateOwnership(
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
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::Ownership {},
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
fn process_on_idle_not_supported() {
    let mut deps = mock_dependencies(&[]);

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::lsm_share_bond_provider::ContractError::MessageIsNotSupported {}
    );
}

#[test]
fn execute_bond() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[Coin::new(100u128, "base_denom")]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);

    assert_eq!(
        response,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-native-bond-provider-bond")
                    .add_attributes(vec![("received_funds", "100base_denom"),])
            )
            .add_submessages(vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "staker_contract".to_string(),
                amount: vec![Coin::new(100u128, "base_denom")],
            }))])
    );
}

#[test]
fn execute_bond_wrong_denom() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[Coin::new(100u128, "wrong_denom")]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::lsm_share_bond_provider::ContractError::InvalidDenom {}
    );
}

#[test]
fn execute_bond_no_funds() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::lsm_share_bond_provider::ContractError::PaymentError(
            PaymentError::NoFunds {}
        )
    );
}

#[test]
fn execute_bond_multiple_denoms() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::lsm_share_bond_provider::Config {
                puppeteer_contract: Addr::unchecked("puppeteer_contract"),
                core_contract: Addr::unchecked("core_contract"),
                validators_set_contract: Addr::unchecked("validators_set_contract"),
                transfer_channel_id: "transfer_channel_id".to_string(),
                lsm_redeem_threshold: 100u64,
                lsm_redeem_maximum_interval: 200u64,
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "core",
            &[
                Coin::new(100u128, "base_denom"),
                Coin::new(100u128, "second_denom"),
            ],
        ),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::lsm_share_bond_provider::ContractError::PaymentError(
            PaymentError::MultipleDenoms {}
        )
    );
}
