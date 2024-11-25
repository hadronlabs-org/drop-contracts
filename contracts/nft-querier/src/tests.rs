use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, NftState, QueryMsg},
    state::{Config, CONFIG},
};
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Api, Empty, Event, Response, Uint128,
};
use cw721::{NftInfoResponse, OwnerOfResponse};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::msg::withdrawal_voucher::Extension;

pub type Cw721VoucherContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty, Empty, Empty>;

#[test]
fn test_execute_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let factory_addr = deps.api.addr_validate("factory_contract").unwrap();
    let res = crate::contract::instantiate(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        InstantiateMsg {
            factory_contract: factory_addr,
        },
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-nft-querier-instantiate")
                .add_attribute("owner", "owner")
        )
    );

    let query_res: cw_ownable::Ownership<String> = from_json(
        crate::contract::query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap(),
    )
    .unwrap();
    assert_eq!(query_res.owner.unwrap(), "owner");
}

#[test]
fn test_execute_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let new_factory_contract = deps.api.addr_validate("new_factory_contract").unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let err = crate::contract::execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("somebody", &[]),
        ExecuteMsg::UpdateConfig {
            new_config: Config {
                factory_contract: new_factory_contract,
            },
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
    )
}

#[test]
fn test_execute_update_config() {
    let mut deps = mock_dependencies(&[]);
    let old_factory_contract = deps.api.addr_validate("old_factory_contract").unwrap();
    let new_factory_contract = deps.api.addr_validate("new_factory_contract").unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();

    CONFIG
        .save(
            deps_mut.storage,
            &Config {
                factory_contract: old_factory_contract,
            },
        )
        .unwrap();

    let res = crate::contract::execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            new_config: Config {
                factory_contract: new_factory_contract,
            },
        },
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-nft-querier-execute-update-config")
                .add_attributes(vec![
                    attr("action", "update-config"),
                    attr("factory_contract", "new_factory_contract")
                ])
        )
    );
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    let factory_contract = deps.api.addr_validate("factory_contract").unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                factory_contract: factory_contract.clone(),
            },
        )
        .unwrap();

    let res: Config =
        from_json(crate::contract::query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap())
            .unwrap();
    assert_eq!(res, Config { factory_contract });
}

#[test]
fn test_query_nft_state_ready() {
    let mut deps = mock_dependencies(&[]);
    let factory_contract = deps.api.addr_validate("factory_contract").unwrap();
    CONFIG
        .save(deps.as_mut().storage, &Config { factory_contract })
        .unwrap();
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
            to_json_binary(&drop_staking_base::state::factory::State {
                token_contract: "token_contract".to_string(),
                core_contract: "core_contract".to_string(),
                puppeteer_contract: "puppeteer_contract".to_string(),
                staker_contract: "staker_contract".to_string(),
                withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
                withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
                strategy_contract: "strategy_contract".to_string(),
                validators_set_contract: "validators_set_contract".to_string(),
                distribution_contract: "distribution_contract".to_string(),
                rewards_manager_contract: "rewards_manager_contract".to_string(),
                rewards_pump_contract: "rewards_pump_contract".to_string(),
                splitter_contract: "splitter_contract".to_string(),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("withdrawal_voucher_contract", |_| {
            to_json_binary(&cw721::AllNftInfoResponse {
                access: OwnerOfResponse {
                    owner: "owner".to_string(),
                    approvals: vec![],
                },
                info: NftInfoResponse {
                    token_uri: None,
                    extension: drop_staking_base::state::withdrawal_voucher::Metadata {
                        name: "name".to_string(),
                        description: None,
                        attributes: None,
                        batch_id: "0".to_string(),
                        amount: Uint128::zero(),
                    },
                },
            })
            .unwrap()
        });
    deps.querier.add_wasm_query_response("core_contract", |_| {
        to_json_binary(&drop_staking_base::state::core::UnbondBatch {
            total_dasset_amount_to_withdraw: Uint128::zero(),
            expected_native_asset_amount: Uint128::zero(),
            expected_release_time: 0u64,
            total_unbond_items: 0u64,
            status: drop_staking_base::state::core::UnbondBatchStatus::Withdrawn,
            slashing_effect: None,
            unbonded_amount: None,
            withdrawn_amount: None,
            status_timestamps: drop_staking_base::state::core::UnbondBatchStatusTimestamps {
                new: 0u64,
                unbond_requested: None,
                unbond_failed: None,
                unbonding: None,
                withdrawing: None,
                withdrawn: None,
                withdrawing_emergency: None,
                withdrawn_emergency: None,
            },
        })
        .unwrap()
    });

    let res: NftState = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::NftState {
                nft_id: "nft".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(res, NftState::Ready);
}

#[test]
fn test_query_nft_state_unready() {
    let mut deps = mock_dependencies(&[]);
    let factory_contract = deps.api.addr_validate("factory_contract").unwrap();

    CONFIG
        .save(deps.as_mut().storage, &Config { factory_contract })
        .unwrap();
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
            to_json_binary(&drop_staking_base::state::factory::State {
                token_contract: "token_contract".to_string(),
                core_contract: "core_contract".to_string(),
                puppeteer_contract: "puppeteer_contract".to_string(),
                staker_contract: "staker_contract".to_string(),
                withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
                withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
                strategy_contract: "strategy_contract".to_string(),
                validators_set_contract: "validators_set_contract".to_string(),
                distribution_contract: "distribution_contract".to_string(),
                rewards_manager_contract: "rewards_manager_contract".to_string(),
                rewards_pump_contract: "rewards_pump_contract".to_string(),
                splitter_contract: "splitter_contract".to_string(),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("withdrawal_voucher_contract", |_| {
            to_json_binary(&cw721::AllNftInfoResponse {
                access: OwnerOfResponse {
                    owner: "owner".to_string(),
                    approvals: vec![],
                },
                info: NftInfoResponse {
                    token_uri: None,
                    extension: drop_staking_base::state::withdrawal_voucher::Metadata {
                        name: "name".to_string(),
                        description: None,
                        attributes: None,
                        batch_id: "0".to_string(),
                        amount: Uint128::zero(),
                    },
                },
            })
            .unwrap()
        });
    deps.querier.add_wasm_query_response("core_contract", |_| {
        to_json_binary(&drop_staking_base::state::core::UnbondBatch {
            total_dasset_amount_to_withdraw: Uint128::zero(),
            expected_native_asset_amount: Uint128::zero(),
            expected_release_time: 0u64,
            total_unbond_items: 0u64,
            status: drop_staking_base::state::core::UnbondBatchStatus::New,
            slashing_effect: None,
            unbonded_amount: None,
            withdrawn_amount: None,
            status_timestamps: drop_staking_base::state::core::UnbondBatchStatusTimestamps {
                new: 0u64,
                unbond_requested: None,
                unbond_failed: None,
                unbonding: None,
                withdrawing: None,
                withdrawn: None,
                withdrawing_emergency: None,
                withdrawn_emergency: None,
            },
        })
        .unwrap()
    });

    let res: NftState = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::NftState {
                nft_id: "nft".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(res, NftState::Unready);
}

#[test]
fn test_query_nft_state_unknown_nft_id() {
    let mut deps = mock_dependencies(&[]);
    let factory_contract = deps.api.addr_validate("factory_contract").unwrap();
    CONFIG
        .save(deps.as_mut().storage, &Config { factory_contract })
        .unwrap();
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
            to_json_binary(&drop_staking_base::state::factory::State {
                token_contract: "token_contract".to_string(),
                core_contract: "core_contract".to_string(),
                puppeteer_contract: "puppeteer_contract".to_string(),
                staker_contract: "staker_contract".to_string(),
                withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
                withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
                strategy_contract: "strategy_contract".to_string(),
                validators_set_contract: "validators_set_contract".to_string(),
                distribution_contract: "distribution_contract".to_string(),
                rewards_manager_contract: "rewards_manager_contract".to_string(),
                rewards_pump_contract: "rewards_pump_contract".to_string(),
                splitter_contract: "splitter_contract".to_string(),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("withdrawal_voucher_contract", move |_| {
            to_json_binary(&cosmwasm_std::Binary::from(
                Cw721VoucherContract::default()
                    .query(
                        mock_dependencies(&[]).as_ref().into_empty(),
                        mock_env(),
                        cw721_base::QueryMsg::AllNftInfo {
                            token_id: "wrong_token_id".to_string(),
                            include_expired: None,
                        },
                    )
                    .unwrap_err()
                    .to_string()
                    .as_bytes(),
            ))
            .unwrap()
        });

    let res = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::NftState {
            nft_id: "nft".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(res, crate::error::ContractError::UnknownNftId {})
}

#[test]
fn test_query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            crate::msg::QueryMsg::Ownership {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(cosmwasm_std::Addr::unchecked("owner".to_string())),
            pending_expiry: None,
            pending_owner: None
        }
    );
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    crate::contract::execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: "new_owner".to_string(),
            expiry: Some(cw_ownable::Expiration::Never {}),
        }),
    )
    .unwrap();
    crate::contract::execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("new_owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership {}),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            crate::msg::QueryMsg::Ownership {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(cosmwasm_std::Addr::unchecked("new_owner".to_string())),
            pending_expiry: None,
            pending_owner: None
        }
    );
}
