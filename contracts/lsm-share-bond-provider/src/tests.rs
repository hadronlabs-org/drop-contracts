use std::borrow::BorrowMut;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info, MockApi},
    to_json_binary, Addr, Coin, Decimal, Decimal256, Event, MemoryStorage, OwnedDeps, Response,
    Timestamp, Uint128,
};
use cw_ownable::{Action, Ownership};
use cw_utils::PaymentError;
use drop_helpers::testing::{mock_dependencies, WasmMockQuerier};
use drop_puppeteer_base::state::{Delegations, DropDelegation};
use drop_staking_base::{
    msg::puppeteer::DelegationsResponse,
    state::lsm_share_bond_provider::{
        Config, ConfigOptional, CONFIG, LAST_LSM_REDEEM, TOTAL_LSM_SHARES,
    },
};
use neutron_sdk::bindings::query::NeutronQuery;

use crate::contract::check_denom::{DenomTrace, QueryDenomTraceResponse};

use prost::Message;

fn get_default_config(lsm_redeem_threshold: u64, lsm_redeem_maximum_interval: u64) -> Config {
    Config {
        puppeteer_contract: Addr::unchecked("puppeteer_contract"),
        core_contract: Addr::unchecked("core_contract"),
        validators_set_contract: Addr::unchecked("validators_set_contract"),
        transfer_channel_id: "transfer_channel_id".to_string(),
        lsm_redeem_threshold,
        lsm_redeem_maximum_interval,
    }
}

#[cw_serde]
pub struct QueryDenomTraceRequest {
    pub hash: String,
}

fn lsm_denom_query_config(
    deps: &mut OwnedDeps<MemoryStorage, MockApi, WasmMockQuerier, NeutronQuery>,
) {
    deps.querier.add_stargate_query_response(
        "/ibc.applications.transfer.v1.Query/DenomTrace",
        |request| {
            let request =
                cosmos_sdk_proto::ibc::applications::transfer::v1::QueryDenomTraceRequest::decode(
                    request.as_slice(),
                )
                .unwrap();
            if request.hash == "lsm_denom_1" {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "transfer/transfer_channel_id".to_string(),
                    },
                })
                .unwrap()
            } else {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "wrong_denom".to_string(),
                    },
                })
                .unwrap()
            }
        },
    );
    deps.querier
        .add_wasm_query_response("validators_set_contract", |request| {
            let request =
                from_json::<drop_staking_base::msg::validatorset::QueryMsg>(request).unwrap();
            if let drop_staking_base::msg::validatorset::QueryMsg::Validator { valoper } = request {
                assert_eq!(valoper, "valoper12345");
                to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                    validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                        valoper_address: "valoper12345".to_string(),
                        weight: 1u64,
                        last_processed_remote_height: None,
                        last_processed_local_height: None,
                        last_validated_height: None,
                        last_commission_in_range: None,
                        uptime: Decimal::one(),
                        tombstone: false,
                        jailed_number: None,
                        init_proposal: None,
                        total_passed_proposals: 0u64,
                        total_voted_proposals: 0u64,
                    }),
                })
                .unwrap()
            } else {
                unimplemented!()
            }
        });

    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![DropDelegation {
                        delegator: Addr::unchecked("delegator"),
                        validator: "valoper12345".to_string(),
                        amount: Coin::new(1000, "remote_denom".to_string()),
                        share_ratio: Decimal256::one(),
                    }],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
}

#[test]
fn test_instantiate() {
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

    assert_eq!(config, get_default_config(100u64, 200u64));

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
fn test_update_config_wrong_owner() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(deps.as_mut().storage, &get_default_config(100u64, 200u64))
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
fn test_update_config_ok() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    drop_staking_base::state::lsm_share_bond_provider::CONFIG
        .save(deps.as_mut().storage, &get_default_config(100u64, 200u64))
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
fn test_update_ownership() {
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
fn test_process_on_idle_supported() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();

    CONFIG
        .save(deps_mut.storage, &get_default_config(100u64, 200u64))
        .unwrap();
    LAST_LSM_REDEEM.save(deps_mut.storage, &0).unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap();

    assert_eq!(
        response,
        Response::new()
            .add_event(Event::new(
                "crates.io:drop-staking__drop-lsm-share-bond-provider-update_config"
            ))
            .add_attributes(vec![
                attr("action", "process_on_idle"),
                attr("knot", "036"),
                attr("knot", "041")
            ])
    );
}

#[test]
fn test_execute_bond() {
    let mut deps = mock_dependencies(&[]);
    lsm_denom_query_config(deps.borrow_mut());

    let deps_mut = deps.as_mut();

    CONFIG
        .save(deps_mut.storage, &get_default_config(100u64, 200u64))
        .unwrap();

    TOTAL_LSM_SHARES.save(deps_mut.storage, &0).unwrap();

    let pending_lsm_shares = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::PendingLSMShares {},
    )
    .unwrap();

    assert_eq!(
        pending_lsm_shares,
        to_json_binary::<Vec<(String, (String, Uint128, Uint128))>>(&vec![]).unwrap()
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[Coin::new(100u128, "lsm_denom_1")]),
        drop_staking_base::msg::lsm_share_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-lsm-share-bond-provider-bond").add_attributes(
                vec![
                    ("received_funds", "100lsm_denom_1"),
                    ("bonded_funds", "100valoper12345/1")
                ]
            )
        )
    );

    let pending_lsm_shares = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::PendingLSMShares {},
    )
    .unwrap();

    assert_eq!(
        pending_lsm_shares,
        to_json_binary::<Vec<(String, (String, Uint128, Uint128))>>(&vec![(
            "lsm_denom_1".to_string(),
            (
                "valoper12345/1".to_string(),
                Uint128::from(100u64),
                Uint128::from(100u64)
            )
        )])
        .unwrap()
    );
}

#[test]
fn test_execute_bond_wrong_denom() {
    let mut deps = mock_dependencies(&[]);
    lsm_denom_query_config(deps.borrow_mut());

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
fn test_execute_bond_no_funds() {
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
fn test_execute_bond_multiple_denoms() {
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

mod query {
    use super::*;

    #[test]
    fn test_config() {
        let mut deps = mock_dependencies(&[]);
        drop_staking_base::state::lsm_share_bond_provider::CONFIG
            .save(deps.as_mut().storage, &get_default_config(100u64, 200u64))
            .unwrap();

        let response = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::Config {},
        )
        .unwrap();
        assert_eq!(
            response,
            to_json_binary(&get_default_config(100u64, 200u64)).unwrap()
        );
    }

    #[test]
    fn test_token_amount_wrong_denom() {
        let mut deps = mock_dependencies(&[]);
        lsm_denom_query_config(deps.borrow_mut());

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
    fn test_can_process_idle_with_enough_interval() {
        let mut deps = mock_dependencies(&[]);
        let deps_mut = deps.as_mut();

        let env = mock_env();
        let lsm_redeem_maximum_interval = 100;

        drop_staking_base::state::lsm_share_bond_provider::CONFIG
            .save(
                deps_mut.storage,
                &get_default_config(2u64, lsm_redeem_maximum_interval),
            )
            .unwrap();

        LAST_LSM_REDEEM
            .save(
                deps_mut.storage,
                &(env.block.time.seconds() - lsm_redeem_maximum_interval * 2),
            )
            .unwrap();

        drop_staking_base::state::lsm_share_bond_provider::LSM_SHARES_TO_REDEEM
            .save(
                deps_mut.storage,
                "lsm_denom_1".to_string(),
                &("lsm_denom_1".to_string(), Uint128::one(), Uint128::one()),
            )
            .unwrap();

        let response = crate::contract::query(
            deps.as_ref(),
            env,
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanProcessOnIdle {},
        )
        .unwrap();
        assert_eq!(response, to_json_binary(&true).unwrap());
    }

    #[test]
    fn test_can_process_false_below_threshold() {
        let mut deps = mock_dependencies(&[]);
        let deps_mut = deps.as_mut();

        let env = mock_env();

        drop_staking_base::state::lsm_share_bond_provider::CONFIG
            .save(deps_mut.storage, &get_default_config(100u64, 200u64))
            .unwrap();

        LAST_LSM_REDEEM
            .save(deps_mut.storage, &(env.block.time.seconds()))
            .unwrap();

        drop_staking_base::state::lsm_share_bond_provider::LSM_SHARES_TO_REDEEM
            .save(
                deps_mut.storage,
                "lsm_denom_1".to_string(),
                &("lsm_denom_1".to_string(), Uint128::one(), Uint128::one()),
            )
            .unwrap();

        let response = crate::contract::query(
            deps.as_ref(),
            env,
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanProcessOnIdle {},
        )
        .unwrap();
        assert_eq!(response, to_json_binary(&false).unwrap());
    }

    #[test]
    fn test_ownership() {
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
    fn test_can_bond_ok() {
        let mut deps = mock_dependencies(&[]);
        lsm_denom_query_config(deps.borrow_mut());

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
                denom: "lsm_denom_1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(can_bond, to_json_binary(&true).unwrap());
    }

    #[test]
    fn test_can_bond_false() {
        let mut deps = mock_dependencies(&[]);
        lsm_denom_query_config(deps.borrow_mut());

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
    fn test_pending_lsm_shares() {
        let mut deps = mock_dependencies(&[]);

        let deps_mut = deps.as_mut();

        drop_staking_base::state::lsm_share_bond_provider::PENDING_LSM_SHARES
            .save(
                deps_mut.storage,
                "lsm_denom_1".to_string(),
                &("lsm_denom_1".to_string(), Uint128::one(), Uint128::one()),
            )
            .unwrap();

        let pending_lsm_shares = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::PendingLSMShares {},
        )
        .unwrap();

        assert_eq!(
            pending_lsm_shares,
            to_json_binary(&vec![(
                "lsm_denom_1".to_string(),
                ("lsm_denom_1".to_string(), Uint128::one(), Uint128::one())
            )])
            .unwrap()
        );
    }

    #[test]
    fn test_lsm_shares_to_redeem() {
        let mut deps = mock_dependencies(&[]);

        let deps_mut = deps.as_mut();

        drop_staking_base::state::lsm_share_bond_provider::LSM_SHARES_TO_REDEEM
            .save(
                deps_mut.storage,
                "lsm_denom_1".to_string(),
                &("lsm_denom_1".to_string(), Uint128::one(), Uint128::one()),
            )
            .unwrap();

        let lsm_shares_to_redeem = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::LSMSharesToRedeem {},
        )
        .unwrap();

        assert_eq!(
            lsm_shares_to_redeem,
            to_json_binary(&vec![(
                "lsm_denom_1".to_string(),
                ("lsm_denom_1".to_string(), Uint128::one(), Uint128::one())
            )])
            .unwrap()
        );
    }

    #[test]
    fn test_total_lsm_shares() {
        let mut deps = mock_dependencies(&[]);

        let deps_mut = deps.as_mut();

        drop_staking_base::state::lsm_share_bond_provider::TOTAL_LSM_SHARES
            .save(deps_mut.storage, &100u128)
            .unwrap();

        let can_bond = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TotalLSMShares {},
        )
        .unwrap();

        assert_eq!(can_bond, to_json_binary(&100u128).unwrap());
    }

    #[test]
    fn test_can_process_idle_false_without_shares() {
        let mut deps = mock_dependencies(&[]);
        let deps_mut = deps.as_mut();

        CONFIG
            .save(deps_mut.storage, &get_default_config(100u64, 200u64))
            .unwrap();

        LAST_LSM_REDEEM.save(deps_mut.storage, &0).unwrap();

        let response = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanProcessOnIdle {},
        )
        .unwrap();
        assert_eq!(response, to_json_binary(&false).unwrap());
    }

    #[test]
    fn test_can_process_idle_with_pending_shares() {
        let mut deps = mock_dependencies(&[]);
        let deps_mut = deps.as_mut();

        drop_staking_base::state::lsm_share_bond_provider::CONFIG
            .save(deps_mut.storage, &get_default_config(100u64, 200u64))
            .unwrap();

        drop_staking_base::state::lsm_share_bond_provider::PENDING_LSM_SHARES
            .save(
                deps_mut.storage,
                "lsm_denom_1".to_string(),
                &("lsm_denom_1".to_string(), Uint128::one(), Uint128::one()),
            )
            .unwrap();

        let response = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanProcessOnIdle {},
        )
        .unwrap();
        assert_eq!(response, to_json_binary(&true).unwrap());
    }

    #[test]
    fn test_can_process_idle_with_enough_redeem_shares() {
        let mut deps = mock_dependencies(&[]);
        let deps_mut = deps.as_mut();

        drop_staking_base::state::lsm_share_bond_provider::CONFIG
            .save(deps_mut.storage, &get_default_config(2u64, 200u64))
            .unwrap();

        LAST_LSM_REDEEM.save(deps_mut.storage, &0).unwrap();

        drop_staking_base::state::lsm_share_bond_provider::LSM_SHARES_TO_REDEEM
            .save(
                deps_mut.storage,
                "lsm_denom_1".to_string(),
                &("lsm_denom_1".to_string(), Uint128::one(), Uint128::one()),
            )
            .unwrap();

        drop_staking_base::state::lsm_share_bond_provider::LSM_SHARES_TO_REDEEM
            .save(
                deps_mut.storage,
                "lsm_denom_2".to_string(),
                &("lsm_denom_2".to_string(), Uint128::one(), Uint128::one()),
            )
            .unwrap();

        let response = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::CanProcessOnIdle {},
        )
        .unwrap();
        assert_eq!(response, to_json_binary(&true).unwrap());
    }

    #[test]
    fn test_token_amount() {
        let mut deps = mock_dependencies(&[]);
        lsm_denom_query_config(deps.borrow_mut());

        CONFIG
            .save(deps.as_mut().storage, &get_default_config(100u64, 200u64))
            .unwrap();

        let token_amount = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TokenAmount {
                coin: Coin {
                    denom: "lsm_denom_1".to_string(),
                    amount: 100u128.into(),
                },
                exchange_rate: Decimal::one(),
            },
        )
        .unwrap();

        assert_eq!(token_amount, to_json_binary(&100u128).unwrap());
    }

    #[test]
    fn test_token_amount_half() {
        let mut deps = mock_dependencies(&[]);
        lsm_denom_query_config(deps.borrow_mut());

        CONFIG
            .save(deps.as_mut().storage, &get_default_config(100u64, 200u64))
            .unwrap();

        let token_amount = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TokenAmount {
                coin: Coin {
                    denom: "lsm_denom_1".to_string(),
                    amount: 100u128.into(),
                },
                exchange_rate: Decimal::from_atomics(Uint128::from(5u64), 1).unwrap(),
            },
        )
        .unwrap();

        assert_eq!(token_amount, to_json_binary(&200u128).unwrap());
    }

    #[test]
    fn test_token_amount_above_one() {
        let mut deps = mock_dependencies(&[]);
        lsm_denom_query_config(deps.borrow_mut());

        CONFIG
            .save(deps.as_mut().storage, &get_default_config(100u64, 200u64))
            .unwrap();

        let token_amount = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::lsm_share_bond_provider::QueryMsg::TokenAmount {
                coin: Coin {
                    denom: "lsm_denom_1".to_string(),
                    amount: 100u128.into(),
                },
                exchange_rate: Decimal::from_atomics(Uint128::from(11u64), 1).unwrap(),
            },
        )
        .unwrap();

        assert_eq!(token_amount, to_json_binary(&90u128).unwrap());
    }
}

mod check_denom {

    use drop_staking_base::error::lsm_share_bond_provider::ContractError;

    use crate::contract::check_denom::{DenomData, DenomTrace, QueryDenomTraceResponse};

    use super::*;

    #[test]
    fn test_invalid_port() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "icahost/transfer_channel_id".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(100, 200),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn test_invalid_channel() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "transfer/unknown_channel".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(100, 200),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn test_invalid_port_and_channel() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "icahost/unknown_channel".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(100, 200),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn test_not_an_lsm_share() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "unknown_denom".to_string(),
                        path: "transfer/transfer_channel_id".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(100, 200),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn test_unknown_validator() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper98765/1".to_string(),
                        path: "transfer/transfer_channel_id".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let query_called = std::rc::Rc::new(std::cell::RefCell::new(false));
        let query_called_cb = std::rc::Rc::clone(&query_called);
        deps.querier
            .add_wasm_query_response("validators_set_contract", move |request| {
                let request =
                    from_json::<drop_staking_base::msg::validatorset::QueryMsg>(request).unwrap();
                if let drop_staking_base::msg::validatorset::QueryMsg::Validator { valoper } =
                    request
                {
                    assert_eq!(valoper, "valoper98765");

                    query_called_cb.replace(true);
                    to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                        validator: None,
                    })
                    .unwrap()
                } else {
                    unimplemented!()
                }
            });
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(100, 200),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
        assert!(*query_called.borrow());
    }

    #[test]
    fn test_invalid_validator_index() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1/2".to_string(),
                        path: "transfer/transfer_channel_id".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(100, 200),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn test_known_validator() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "transfer/transfer_channel_id".to_string(),
                    },
                })
                .unwrap()
            },
        );
        deps.querier
            .add_wasm_query_response("validators_set_contract", |request| {
                let request =
                    from_json::<drop_staking_base::msg::validatorset::QueryMsg>(request).unwrap();
                if let drop_staking_base::msg::validatorset::QueryMsg::Validator { valoper } =
                    request
                {
                    assert_eq!(valoper, "valoper12345");
                    to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                        validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                            valoper_address: "valoper12345".to_string(),
                            weight: 1u64,
                            last_processed_remote_height: None,
                            last_processed_local_height: None,
                            last_validated_height: None,
                            last_commission_in_range: None,
                            uptime: Decimal::one(),
                            tombstone: false,
                            jailed_number: None,
                            init_proposal: None,
                            total_passed_proposals: 0u64,
                            total_voted_proposals: 0u64,
                        }),
                    })
                    .unwrap()
                } else {
                    unimplemented!()
                }
            });
        let denom_data = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(100, 200),
        )
        .unwrap();
        assert_eq!(
            denom_data,
            DenomData {
                remote_denom: "valoper12345/1".to_string(),
                validator: "valoper12345".to_string()
            }
        );
    }
}
