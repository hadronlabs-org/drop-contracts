use crate::contract::{execute, query};
use cosmwasm_std::{
    from_json,
    testing::{message_info, mock_env, MockApi, MockStorage},
    to_json_binary, Addr, Coin, CosmosMsg, Decimal, Decimal256, Event, OwnedDeps, Response, SubMsg,
    Timestamp, Uint128, WasmMsg,
};
use drop_helpers::{
    pause::PauseError,
    testing::{mock_dependencies, mock_state_query, WasmMockQuerier},
};
use drop_puppeteer_base::msg::TransferReadyBatchesMsg;
use drop_staking_base::{
    error::core::ContractError,
    msg::{
        core::{ExecuteMsg, FailedBatchResponse, InstantiateMsg, QueryMsg},
        puppeteer::{BalancesResponse, DelegationsResponse},
        strategy::QueryMsg as StrategyQueryMsg,
    },
    state::{
        core::{
            unbond_batches_map, Config, ConfigOptional, ContractState, Pause, UnbondBatch,
            UnbondBatchStatus, UnbondBatchStatusTimestamps, BOND_HOOKS, BOND_PROVIDERS, CONFIG,
            FAILED_BATCH_ID, FSM, LAST_ICA_CHANGE_HEIGHT, LAST_IDLE_CALL, LAST_PUPPETEER_RESPONSE,
            LD_DENOM, MAX_BOND_PROVIDERS, PAUSE, UNBOND_BATCH_ID,
        },
        puppeteer::{Delegations, DropDelegation},
    },
};
use neutron_sdk::{bindings::query::NeutronQuery, interchain_queries::v045::types::Balances};
use std::{collections::HashMap, vec};

fn get_default_config(
    api: MockApi,
    idle_min_interval: u64,
    unbonding_safe_period: u64,
    unbond_batch_switch_time: u64,
) -> Config {
    Config {
        factory_contract: api.addr_make("factory_contract"),
        base_denom: "base_denom".to_string(),
        remote_denom: "remote_denom".to_string(),
        idle_min_interval,
        unbonding_period: 60,
        unbonding_safe_period,
        unbond_batch_switch_time,
        pump_ica_address: Some(api.addr_make("pump_address").to_string()),
        emergency_address: None,
        icq_update_delay: 5,
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
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    deps.querier
        .add_wasm_query_response(api.addr_make("token_contract").as_str(), move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&drop_staking_base::msg::token::ConfigResponse {
                    factory_contract: api.addr_make("factory_contract").to_string(),
                    denom: "ld_denom".to_string(),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response("old_token_contract", |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&drop_staking_base::msg::token::ConfigResponse {
                    factory_contract: "factory_contract".to_string(),
                    denom: "ld_denom".to_string(),
                })
                .unwrap(),
            )
        });

    mock_state_query(&mut deps);
    let env = mock_env();
    let info = message_info(&api.addr_make("admin"), &[]);
    let mut deps_mut = deps.as_mut();
    crate::contract::instantiate(
        deps_mut.branch(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            factory_contract: api.addr_make("factory_contract").to_string(),
            base_denom: "old_base_denom".to_string(),
            remote_denom: "old_remote_denom".to_string(),
            idle_min_interval: 12,
            unbonding_period: 20,
            unbonding_safe_period: 120,
            unbond_batch_switch_time: 2000,
            pump_ica_address: Some(api.addr_make("old_pump_address").to_string()),
            emergency_address: Some(api.addr_make("old_emergency_address").to_string()),
            owner: api.addr_make("admin").to_string(),
            icq_update_delay: 5,
        },
    )
    .unwrap();
    assert_eq!(
        LD_DENOM.may_load(deps_mut.storage).unwrap(),
        Some("ld_denom".to_string())
    );

    let new_config = ConfigOptional {
        factory_contract: Some(api.addr_make("new_factory_contract").to_string()),
        base_denom: Some("new_base_denom".to_string()),
        remote_denom: Some("new_remote_denom".to_string()),
        idle_min_interval: Some(2),
        unbonding_period: Some(120),
        unbonding_safe_period: Some(20),
        unbond_batch_switch_time: Some(12000),
        pump_ica_address: Some(api.addr_make("new_pump_address").to_string()),
        rewards_receiver: Some(api.addr_make("new_rewards_receiver").to_string()),
        emergency_address: Some(api.addr_make("new_emergency_address").to_string()),
    };
    let expected_config = Config {
        factory_contract: api.addr_make("new_factory_contract"),
        base_denom: "new_base_denom".to_string(),
        remote_denom: "new_remote_denom".to_string(),
        idle_min_interval: 2,
        unbonding_period: 120,
        unbonding_safe_period: 20,
        unbond_batch_switch_time: 12000,
        pump_ica_address: Some(api.addr_make("new_pump_address").to_string()),
        emergency_address: Some(api.addr_make("new_emergency_address").to_string()),
        icq_update_delay: 5,
    };

    let res = execute(
        deps_mut,
        env.clone(),
        info,
        ExecuteMsg::UpdateConfig {
            new_config: Box::new(new_config),
        },
    );
    assert!(res.is_ok());
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, expected_config);
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    let config = get_default_config(api, 1000, 10, 6000);
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    assert_eq!(
        from_json::<Config>(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap())
            .unwrap(),
        config
    );
}

#[test]
fn query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let owner_address = deps.api.addr_make("owner");
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some(owner_address.as_str()))
        .unwrap();
    assert_eq!(
        from_json::<String>(&query(deps.as_ref(), mock_env(), QueryMsg::Owner {}).unwrap())
            .unwrap(),
        owner_address.as_str(),
    );
}

#[test]
fn test_bond_provider_has_any_tokens() {
    let mut deps = mock_dependencies(&[]);
    let owner_address = deps.api.addr_make("owner");
    let bond_provider_address = deps.api.addr_make("bond_provider");

    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some(owner_address.as_str()))
        .unwrap();

    deps.querier
        .add_wasm_query_response(bond_provider_address.as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&false).unwrap())
        });

    let error = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&owner_address, &[]),
        ExecuteMsg::RemoveBondProvider {
            bond_provider_address: bond_provider_address.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(error, ContractError::BondProviderBalanceNotEmpty {})
}

#[test]
fn test_execute_add_bond_provider_max_limit_reached() {
    let mut deps = mock_dependencies(&[]);
    let owner_address = deps.api.addr_make("owner");
    let overflow_bond_provider_address = deps.api.addr_make("overflow_bond_provider");

    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some(owner_address.as_str()))
        .unwrap();

    for i in 0..MAX_BOND_PROVIDERS {
        let mut provider: String = "provider".to_string();
        provider.push_str(i.to_string().as_str());

        BOND_PROVIDERS
            .add(deps_mut.storage, Addr::unchecked(provider))
            .unwrap();
    }
    let res = execute(
        deps_mut,
        mock_env(),
        message_info(&owner_address, &[]),
        ExecuteMsg::AddBondProvider {
            bond_provider_address: overflow_bond_provider_address.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::MaxBondProvidersReached {})
}

#[test]
fn test_update_withdrawn_amount() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 10, 6000),
        )
        .unwrap();
    mock_state_query(&mut deps);
    let withdrawn_batch = &UnbondBatch {
        total_dasset_amount_to_withdraw: Uint128::from(1001u128),
        expected_native_asset_amount: Uint128::from(1001u128),
        total_unbond_items: 1,
        status: UnbondBatchStatus::Withdrawn,
        expected_release_time: 9000,
        slashing_effect: None,
        unbonded_amount: None,
        withdrawn_amount: None,
        status_timestamps: get_default_unbond_batch_status_timestamps(),
    };

    let unbonding_batch = &UnbondBatch {
        total_dasset_amount_to_withdraw: Uint128::from(2002u128),
        expected_native_asset_amount: Uint128::from(2002u128),
        total_unbond_items: 1,
        status: UnbondBatchStatus::Unbonding,
        expected_release_time: 9000,
        slashing_effect: None,
        unbonded_amount: None,
        withdrawn_amount: None,
        status_timestamps: get_default_unbond_batch_status_timestamps(),
    };

    unbond_batches_map()
        .save(deps.as_mut().storage, 1, withdrawn_batch)
        .unwrap();

    unbond_batches_map()
        .save(deps.as_mut().storage, 0, unbonding_batch)
        .unwrap();

    let withdrawn_res = execute(
        deps.as_mut(),
        mock_env().clone(),
        message_info(&api.addr_make("withdrawal_manager_contract"), &[]),
        ExecuteMsg::UpdateWithdrawnAmount {
            batch_id: 1,
            withdrawn_amount: Uint128::from(1001u128),
        },
    );
    assert!(withdrawn_res.is_ok());

    let new_withdrawn_amount = unbond_batches_map()
        .load(deps.as_mut().storage, 1)
        .unwrap()
        .withdrawn_amount;
    assert_eq!(new_withdrawn_amount, Some(Uint128::from(1001u128)));

    let unbonding_err = execute(
        deps.as_mut(),
        mock_env().clone(),
        message_info(&api.addr_make("withdrawal_manager_contract"), &[]),
        ExecuteMsg::UpdateWithdrawnAmount {
            batch_id: 0,
            withdrawn_amount: Uint128::from(2002u128),
        },
    )
    .unwrap_err();
    assert_eq!(unbonding_err, ContractError::BatchNotWithdrawn {});
}

#[test]
fn test_add_remove_bond_provider() {
    let mut deps = mock_dependencies(&[]);
    let admin_address = deps.api.addr_make("admin");
    let bond_provider_address = deps.api.addr_make("bond_provider");

    mock_state_query(&mut deps);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some(admin_address.as_str()))
        .unwrap();

    let bond_providers = query(deps.as_ref(), mock_env(), QueryMsg::BondProviders {}).unwrap();

    assert_eq!(
        bond_providers,
        to_json_binary::<Vec<(Addr, bool)>>(&vec![]).unwrap()
    );

    deps.querier
        .add_wasm_query_response(bond_provider_address.as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
        });

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin_address, &[]),
        ExecuteMsg::AddBondProvider {
            bond_provider_address: bond_provider_address.to_string(),
        },
    );
    assert_eq!(
        res,
        Ok(Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-core-execute-add_bond_provider")
                .add_attributes(vec![(
                    "bond_provider_address",
                    bond_provider_address.clone()
                )])
        ))
    );

    let bond_providers = query(deps.as_ref(), mock_env(), QueryMsg::BondProviders {}).unwrap();

    assert_eq!(
        bond_providers,
        to_json_binary(&vec![bond_provider_address.clone()]).unwrap()
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&admin_address, &[]),
        ExecuteMsg::RemoveBondProvider {
            bond_provider_address: bond_provider_address.to_string(),
        },
    );
    assert_eq!(
        res,
        Ok(Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-core-execute-remove_bond_provider")
                .add_attributes(vec![(
                    "bond_provider_address",
                    bond_provider_address.clone()
                )])
        ))
    );

    let bond_providers = query(deps.as_ref(), mock_env(), QueryMsg::BondProviders {}).unwrap();

    assert_eq!(
        bond_providers,
        to_json_binary::<Vec<(Addr, bool)>>(&vec![]).unwrap()
    );
}

#[test]
fn test_execute_tick_idle_process_bondig_provider() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);
    BOND_PROVIDERS.init(deps.as_mut().storage).unwrap();

    deps.querier
        .add_wasm_query_response(api.addr_make("lsm_provider_address").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
        });

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });

    BOND_PROVIDERS
        .add(deps.as_mut().storage, api.addr_make("lsm_provider_address"))
        .unwrap();

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 10, 6000),
        )
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_IDLE_CALL.save(deps.as_mut().storage, &0).unwrap();

    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);
    let res = execute(
        deps.as_mut(),
        env,
        message_info(&api.addr_make("admin"), &[]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-tick_idle").add_attributes(
                    vec![
                        ("action", "tick_idle"),
                        ("knot", "000"),
                        ("knot", "002"),
                        ("knot", "003"),
                        ("knot", "036"),
                        (
                            "used_bond_provider",
                            api.addr_make("lsm_provider_address").as_str()
                        ),
                    ]
                )
            )
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("lsm_provider_address").to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::bond_provider::ExecuteMsg::ProcessOnIdle {}
                )
                .unwrap(),
                funds: vec![],
            }))
    );
}

#[test]
fn test_tick_idle_claim_wo_unbond() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances {
                        coins: vec![Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::new(200),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(20),
                })
                .unwrap(),
            )
        });
    deps.querier.add_wasm_query_response(
        api.addr_make("validators_set_contract").as_str(),
        move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&vec![
                    drop_staking_base::state::validatorset::ValidatorInfo {
                        valoper_address: api.addr_make("valoper_address").to_string(),
                        weight: 1,
                        on_top: Uint128::zero(),
                        last_processed_remote_height: None,
                        last_processed_local_height: None,
                        last_validated_height: None,
                        last_commission_in_range: None,
                        uptime: Decimal::one(),
                        tombstone: false,
                        jailed_number: None,
                        init_proposal: None,
                        total_passed_proposals: 0,
                        total_voted_proposals: 0,
                    },
                ])
                .unwrap(),
            )
        },
    );
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![DropDelegation {
                            delegator: api.addr_make("ica_address"),
                            validator: api.addr_make("valoper_address").to_string(),
                            amount: Coin {
                                denom: "remote_denom".to_string(),
                                amount: Uint128::new(100_000),
                            },
                            share_ratio: Decimal256::one(),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 12344u64,
                    timestamp: Timestamp::from_seconds(0),
                })
                .unwrap(),
            )
        });
    let config = get_default_config(api, 1000, 100, 6000);
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_IDLE_CALL.save(deps.as_mut().storage, &0).unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(1000u128),
                expected_native_asset_amount: Uint128::from(1000u128),
                total_unbond_items: 1,
                status: UnbondBatchStatus::Unbonding,
                expected_release_time: 9000,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: get_default_unbond_batch_status_timestamps(),
            },
        )
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(10000);
    let res = execute(
        deps.as_mut(),
        env,
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-tick_idle").add_attributes(
                    vec![
                        ("action", "tick_idle"),
                        ("knot", "000"),
                        ("knot", "002"),
                        ("knot", "003"),
                        ("knot", "004"),
                        ("knot", "005"),
                        ("knot", "007"),
                        ("knot", "009"),
                        ("knot", "010"),
                        ("validators_to_claim", api.addr_make("valoper_address").as_str()),
                        ("knot", "011"),
                        ("knot", "012"),
                        ("state", "claiming"),
                    ]
                )
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("puppeteer_contract").to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                    validators: vec![api.addr_make("valoper_address").to_string()],
                    transfer: None,
                    reply_to: api.addr_make("cosmos2contract").to_string()
                }).unwrap(),
                funds: vec![Coin::new(1000u128, "untrn")],
            })))
    );
}

#[test]
fn test_tick_idle_claim_with_unbond_transfer() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances {
                        coins: vec![Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::new(200),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier.add_wasm_query_response(
        api.addr_make("validators_set_contract").as_str(),
        move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&vec![
                    drop_staking_base::state::validatorset::ValidatorInfo {
                        valoper_address: api.addr_make("valoper_address").to_string(),
                        weight: 1,
                        on_top: Uint128::zero(),
                        last_processed_remote_height: None,
                        last_processed_local_height: None,
                        last_validated_height: None,
                        last_commission_in_range: None,
                        uptime: Decimal::one(),
                        tombstone: false,
                        jailed_number: None,
                        init_proposal: None,
                        total_passed_proposals: 0,
                        total_voted_proposals: 0,
                    },
                ])
                .unwrap(),
            )
        },
    );
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![DropDelegation {
                            delegator: api.addr_make("ica_address"),
                            validator: api.addr_make("valoper_address").to_string(),
                            amount: Coin {
                                denom: "remote_denom".to_string(),
                                amount: Uint128::new(100_000),
                            },
                            share_ratio: Decimal256::one(),
                        }],
                    },
                    remote_height: 12344u64,
                    local_height: 12344u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 6000),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_IDLE_CALL.save(deps.as_mut().storage, &0).unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();

    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(1000u128),
                expected_native_asset_amount: Uint128::from(1000u128),
                total_unbond_items: 1,
                status: UnbondBatchStatus::Unbonding,
                expected_release_time: 90000,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: get_default_unbond_batch_status_timestamps(),
            },
        )
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100000);
    let res = execute(
        deps.as_mut(),
        env,
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
        .add_event(Event::new("crates.io:drop-staking__drop-core-execute-tick_idle").add_attributes(vec![
            ("action", "tick_idle" ),
            ("knot", "000"),
            ("knot", "002"),
            ("knot", "003"),
            ("knot", "004"),
            ("knot", "005"),
            ("knot", "007"),
            ("knot", "008"),
            ("knot", "009"),
            ("knot", "010"),
            ("validators_to_claim",  api.addr_make("valoper_address").as_str()),
            ("knot", "011"),
            ("knot", "012"),
            ("state",  "claiming"),
        ]))
        .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: api.addr_make("puppeteer_contract").to_string(),
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                validators: vec![api.addr_make("valoper_address").to_string()],
                transfer: Some(TransferReadyBatchesMsg{ batch_ids: vec![0u128], emergency: false, amount: Uint128::from(200u128), recipient: api.addr_make("pump_address").to_string() }),
                reply_to: api.addr_make("cosmos2contract").to_string()
            }).unwrap(), funds: vec![Coin::new(1000u128, "untrn")] })))
    );
}

#[test]
fn test_tick_no_puppeteer_response() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::PuppeteerResponseIsNotReceived {}));
}

#[test]
fn test_tick_claiming_error_wo_transfer() {
    // no unbonded batch, no pending transfer for stake, some balance in ICA to stake
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances {
                        coins: vec![Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::new(200),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("staker_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::zero()).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("strategy_contract").as_str(), move |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcDeposit { deposit } => cosmwasm_std::ContractResult::Ok(
                    to_json_binary(&vec![(
                        api.addr_make("valoper_address").to_string(),
                        deposit,
                    )])
                    .unwrap(),
                ),
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Error(
                drop_puppeteer_base::peripheral_hook::ResponseHookErrorMsg {
                    details: "Some error".to_string(),
                    transaction:
                        drop_puppeteer_base::peripheral_hook::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec![api.addr_make("valoper_address").to_string()],
                            denom: "remote_denom".to_string(),
                            transfer: None,
                        },
                },
            ),
        )
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-core-execute-tick_claiming").add_attributes(
                vec![
                    ("action", "tick_claiming"),
                    ("knot", "012"),
                    ("error_on_claiming", "ResponseHookErrorMsg { transaction: ClaimRewardsAndOptionalyTransfer { interchain_account_id: \"ica\", validators: [\"cosmwasm1lf0u3zhca24ws690e5kc8unqjjy9va3ss0cqm0unfgqr7v77r6sq8hnj0p\"], denom: \"remote_denom\", transfer: None }, details: \"Some error\" }"),
                    ("knot", "050"),
                    ("knot", "000"),
                ]
            )
        )
    );
}

#[test]
fn test_tick_claiming_error_with_transfer() {
    // no unbonded batch, no pending transfer for stake, some balance in ICA to stake
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&(
                    Balances {
                        coins: vec![Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::new(200),
                        }],
                    },
                    10u64,
                    Timestamp::from_seconds(90001),
                ))
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("staker_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::zero()).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("strategy_contract").as_str(), move |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcDeposit { deposit } => cosmwasm_std::ContractResult::Ok(
                    to_json_binary(&vec![(
                        api.addr_make("valoper_address").to_string(),
                        deposit,
                    )])
                    .unwrap(),
                ),
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(1000u128),
                expected_native_asset_amount: Uint128::from(1000u128),
                total_unbond_items: 1,
                status: UnbondBatchStatus::Withdrawing,
                expected_release_time: 0,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: UnbondBatchStatusTimestamps {
                    new: 0,
                    unbond_requested: None,
                    unbond_failed: None,
                    unbonding: None,
                    withdrawing: None,
                    withdrawn: None,
                    withdrawing_emergency: None,
                    withdrawn_emergency: None,
                },
            },
        )
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Error(
                drop_puppeteer_base::peripheral_hook::ResponseHookErrorMsg {
                    details: "Some error".to_string(),
                    transaction:
                        drop_puppeteer_base::peripheral_hook::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec![api.addr_make("valoper_address").to_string()],
                            denom: "remote_denom".to_string(),
                            transfer: Some(TransferReadyBatchesMsg {
                                batch_ids: vec![0u128],
                                emergency: false,
                                amount: Uint128::new(123123u128),
                                recipient: api.addr_make("recipient").to_string(),
                            }),
                        },
                },
            ),
        )
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-core-execute-tick_claiming").add_attributes(
                vec![
                    ("action", "tick_claiming"),
                    ("knot", "012"),
                    ("error_on_claiming", "ResponseHookErrorMsg { transaction: ClaimRewardsAndOptionalyTransfer { interchain_account_id: \"ica\", validators: [\"cosmwasm1lf0u3zhca24ws690e5kc8unqjjy9va3ss0cqm0unfgqr7v77r6sq8hnj0p\"], denom: \"remote_denom\", transfer: Some(TransferReadyBatchesMsg { batch_ids: [0], emergency: false, amount: Uint128(123123), recipient: \"cosmwasm1vewsdxxmeraett7ztsaym88jsrv85kzm0xvjg09xqz8aqvjcja0syapxq9\" }) }, details: \"Some error\" }"),
                    ("knot", "050"),
                    ("knot", "000"),
                ]
            )
        )
    );
    let unbond_batch = unbond_batches_map().load(deps.as_mut().storage, 0).unwrap();
    assert_eq!(unbond_batch.status, UnbondBatchStatus::Unbonding);
}

#[test]
fn test_tick_claiming_wo_transfer_unbonding() {
    // no unbonded batch, no pending transfer for stake, no balance on ICA, but we have unbond batch to switch
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances {
                        coins: vec![Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::zero(),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("staker_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::zero()).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances {
                        coins: vec![Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::zero(),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("strategy_contract").as_str(), move |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcWithdraw { withdraw } => cosmwasm_std::ContractResult::Ok(
                    to_json_binary(&vec![(
                        api.addr_make("valoper_address").to_string(),
                        withdraw,
                    )])
                    .unwrap(),
                ),
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(
                drop_puppeteer_base::peripheral_hook::ResponseHookSuccessMsg {
                    local_height: 9u64,
                    remote_height: 9u64,
                    transaction:
                        drop_puppeteer_base::peripheral_hook::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec![api.addr_make("valoper_address").to_string()],
                            denom: "remote_denom".to_string(),
                            transfer: None,
                        },
                },
            ),
        )
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &9u64)
        .unwrap();

    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0u128).unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(1000u128),
                expected_native_asset_amount: Uint128::from(1000u128),
                total_unbond_items: 1,
                status: UnbondBatchStatus::New,
                expected_release_time: 0,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: UnbondBatchStatusTimestamps {
                    new: 0,
                    unbond_requested: None,
                    unbond_failed: None,
                    unbonding: None,
                    withdrawing: None,
                    withdrawn: None,
                    withdrawing_emergency: None,
                    withdrawn_emergency: None,
                },
            },
        )
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100000);
    let res = execute(
        deps.as_mut(),
        env,
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-tick_claiming")
                    .add_attributes(vec![
                        ("action", "tick_claiming"),
                        ("knot", "012"),
                        ("knot", "047"),
                        ("knot", "013"),
                        ("knot", "015"),
                        ("knot", "024"),
                        ("knot", "026"),
                        ("knot", "027"),
                        ("exchange_rate", "1"),
                        ("knot", "045"),
                        ("knot", "049"),
                        ("knot", "046"),
                        ("knot", "028"),
                        ("knot", "029"),
                        ("state", "unbonding")
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("puppeteer_contract").to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
                    items: vec![(
                        api.addr_make("valoper_address").to_string(),
                        Uint128::from(1000u128)
                    )],
                    batch_id: 0u128,
                    reply_to: api.addr_make("cosmos2contract").to_string()
                })
                .unwrap(),
                funds: vec![Coin::new(1000u128, "untrn")],
            })))
    );
    let new_batch_id = UNBOND_BATCH_ID.load(deps.as_mut().storage).unwrap();
    assert_eq!(new_batch_id, 1u128);
    let new_batch = unbond_batches_map().load(deps.as_mut().storage, 1).unwrap();
    assert_eq!(new_batch.status, UnbondBatchStatus::New);
    let old_batch = unbond_batches_map().load(deps.as_mut().storage, 0).unwrap();
    assert_eq!(old_batch.status, UnbondBatchStatus::UnbondRequested);
}

#[test]
fn test_tick_claiming_wo_idle() {
    // no unbonded batch, no pending transfer for stake, no balance on ICA,
    // and no unbond batch to switch, so we go to idle
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances {
                        coins: vec![Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::zero(),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("staker_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::zero()).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("strategy_contract").as_str(), move |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcWithdraw { withdraw } => cosmwasm_std::ContractResult::Ok(
                    to_json_binary(&vec![(
                        api.addr_make("valoper_address").to_string(),
                        withdraw,
                    )])
                    .unwrap(),
                ),
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 60000),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(
                drop_puppeteer_base::peripheral_hook::ResponseHookSuccessMsg {
                    local_height: 9u64,
                    remote_height: 9u64,
                    transaction:
                        drop_puppeteer_base::peripheral_hook::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec![api.addr_make("valoper_address").to_string()],
                            denom: "remote_denom".to_string(),
                            transfer: None,
                        },
                },
            ),
        )
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &9u64)
        .unwrap();

    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0u128).unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(1000u128),
                expected_native_asset_amount: Uint128::from(1000u128),
                total_unbond_items: 1,
                status: UnbondBatchStatus::New,
                expected_release_time: 0,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: UnbondBatchStatusTimestamps {
                    new: 0,
                    unbond_requested: None,
                    unbond_failed: None,
                    unbonding: None,
                    withdrawing: None,
                    withdrawn: None,
                    withdrawing_emergency: None,
                    withdrawn_emergency: None,
                },
            },
        )
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);

    let res = execute(
        deps.as_mut(),
        env,
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-core-execute-tick_claiming").add_attributes(
                vec![
                    ("action", "tick_claiming"),
                    ("knot", "012"),
                    ("knot", "047"),
                    ("knot", "013"),
                    ("knot", "015"),
                    ("knot", "024"),
                    ("knot", "026"),
                    ("knot", "027"),
                    ("knot", "000"),
                    ("state", "idle")
                ]
            )
        )
    );
}

#[test]
fn test_execute_tick_guard_balance_outdated() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &11)
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(
        res,
        Err(ContractError::PuppeteerBalanceOutdated {
            ica_height: 11u64,
            control_height: 10u64
        })
    );
}

#[test]
fn test_execute_tick_guard_delegations_outdated() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &11)
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 12u64,
                    local_height: 12u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(
        res,
        Err(ContractError::PuppeteerDelegationsOutdated {
            ica_height: 11u64,
            control_height: 10u64
        })
    );
}

#[test]
fn test_execute_tick_staking_no_puppeteer_response() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Unbonding)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::PuppeteerResponseIsNotReceived {}));
}

#[test]
fn test_execute_tick_unbonding_no_puppeteer_response() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();

    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&BalancesResponse {
                    balances: Balances { coins: vec![] },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Unbonding)
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[Coin::new(1000u128, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::PuppeteerResponseIsNotReceived {}));
}

#[test]
fn test_bond_wo_receiver() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    BOND_PROVIDERS.init(deps.as_mut().storage).unwrap();

    deps.querier
        .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::from(1000u128)).unwrap())
        });

    BOND_PROVIDERS
        .add(
            deps.as_mut().storage,
            api.addr_make("native_provider_address"),
        )
        .unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    BOND_HOOKS.save(deps.as_mut().storage, &vec![]).unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        message_info(&api.addr_make("some"), &[Coin::new(1000u128, "base_denom")]),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-bond")
                    .add_attribute("action", "bond")
                    .add_attribute("exchange_rate", "1")
                    .add_attribute(
                        "used_bond_provider",
                        api.addr_make("native_provider_address").to_string()
                    )
                    .add_attribute("issue_amount", "1000")
                    .add_attribute("receiver", api.addr_make("some"))
            )
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("native_provider_address").to_string(),
                msg: to_json_binary(&drop_staking_base::msg::bond_provider::ExecuteMsg::Bond {})
                    .unwrap(),
                funds: vec![Coin::new(1000u128, "base_denom")],
            }))
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("token_contract").to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Mint {
                    amount: Uint128::from(1000u128),
                    receiver: api.addr_make("some").to_string()
                })
                .unwrap(),
                funds: vec![],
            }))
    );
}

#[test]
fn test_bond_with_receiver() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);
    BOND_PROVIDERS.init(deps.as_mut().storage).unwrap();

    deps.querier
        .add_wasm_query_response(api.addr_make("native_provider").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("native_provider").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::from(1000u128)).unwrap())
        });

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();

    BOND_PROVIDERS
        .add(
            deps.as_mut().storage,
            api.addr_make("native_provider").clone(),
        )
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    BOND_HOOKS.save(deps.as_mut().storage, &vec![]).unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        message_info(&api.addr_make("some"), &[Coin::new(1000u128, "base_denom")]),
        ExecuteMsg::Bond {
            receiver: Some(api.addr_make("receiver").to_string()),
            r#ref: Some(api.addr_make("ref").to_string()),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-bond")
                    .add_attribute("action", "bond")
                    .add_attribute("exchange_rate", "1")
                    .add_attribute("used_bond_provider", api.addr_make("native_provider"))
                    .add_attribute("issue_amount", "1000")
                    .add_attribute("receiver", api.addr_make("receiver"))
                    .add_attribute("ref", api.addr_make("ref"))
            )
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("native_provider").to_string(),
                msg: to_json_binary(&drop_staking_base::msg::bond_provider::ExecuteMsg::Bond {})
                    .unwrap(),
                funds: vec![Coin::new(1000u128, "base_denom")],
            }))
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("token_contract").to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Mint {
                    amount: Uint128::from(1000u128),
                    receiver: api.addr_make("receiver").to_string()
                })
                .unwrap(),
                funds: vec![],
            }))
    );
}

#[test]
fn check_failed_batch_query_deserialization() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    {
        let result_none = from_json::<FailedBatchResponse>(
            query(
                deps.as_ref(),
                env.clone(),
                drop_staking_base::msg::core::QueryMsg::FailedBatch {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result_none, FailedBatchResponse { response: None });
    }
    {
        FAILED_BATCH_ID.save(&mut deps.storage, &123).unwrap();
        let result_some = from_json::<FailedBatchResponse>(
            query(
                deps.as_ref(),
                env,
                drop_staking_base::msg::core::QueryMsg::FailedBatch {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            result_some,
            FailedBatchResponse {
                response: Some(123)
            }
        );
    }
}

#[test]
fn test_bond_lsm_share_increase_exchange_rate() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "ld_denom".to_string(),
        amount: Uint128::new(1001),
    }]);
    let api = deps.api;

    mock_state_query(&mut deps);

    BOND_PROVIDERS.init(deps.as_mut().storage).unwrap();

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![DropDelegation {
                            delegator: api.addr_make("delegator"),
                            validator: "valoper1".to_string(),
                            amount: Coin::new(1000u128, "remote_denom".to_string()),
                            share_ratio: Decimal256::one(),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![DropDelegation {
                            delegator: api.addr_make("delegator"),
                            validator: "valoper1".to_string(),
                            amount: Coin::new(1000u128, "remote_denom".to_string()),
                            share_ratio: Decimal256::one(),
                        }],
                    },
                    remote_height: 10u64,
                    local_height: 10u64,
                    timestamp: Timestamp::from_seconds(90001),
                })
                .unwrap(),
            )
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::from(100u128)).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
        });
    deps.querier
        .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::from(100500u128)).unwrap())
        });

    BOND_PROVIDERS
        .add(
            deps.as_mut().storage,
            api.addr_make("native_provider_address"),
        )
        .unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    BOND_HOOKS.save(deps.as_mut().storage, &vec![]).unwrap();
    UNBOND_BATCH_ID.save(&mut deps.storage, &0).unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    unbond_batches_map()
        .save(
            &mut deps.storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::zero(),
                expected_native_asset_amount: Uint128::zero(),
                total_unbond_items: 0,
                status: UnbondBatchStatus::New,
                expected_release_time: 0,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: get_default_unbond_batch_status_timestamps(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        message_info(
            &api.addr_make("some"),
            &[Coin::new(100500u128, "lsm_share")],
        ),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    )
    .unwrap();
    let issue_amount = res.events[0]
        .attributes
        .iter()
        .find(|attribute| attribute.key == "issue_amount")
        .unwrap()
        .value
        .parse::<u64>()
        .unwrap();
    assert_eq!(issue_amount, 100500);
}

#[test]
fn test_unbond() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let mut env = mock_env();

    deps.querier
        .add_wasm_query_response("factory_contract", move |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&HashMap::from([
                    ("token_contract", api.addr_make("token_contract").as_str()),
                    (
                        "withdrawal_voucher_contract",
                        api.addr_make("withdrawal_voucher_contract").as_str(),
                    ),
                ]))
                .unwrap(),
            )
        });
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();

    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0u128).unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(0u128),
                expected_native_asset_amount: Uint128::from(0u128),
                total_unbond_items: 0,
                status: UnbondBatchStatus::New,
                expected_release_time: 0,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: get_default_unbond_batch_status_timestamps(),
            },
        )
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(api, 1000, 100, 600),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause::default())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        message_info(
            &api.addr_make("some_sender"),
            &[Coin::new(1000u128, "ld_denom")],
        ),
        ExecuteMsg::Unbond {},
    )
    .unwrap();
    let unbond_batch = unbond_batches_map().load(deps.as_ref().storage, 0).unwrap();
    let extension = Some(
        drop_staking_base::state::withdrawal_voucher::NftExtensionMsg {
            description: Some("Withdrawal voucher".into()),
            name: "LDV voucher".to_string(),
            batch_id: "0".to_string(),
            amount: Uint128::from(1000u128),
            attributes: Some(vec![
                drop_staking_base::state::withdrawal_voucher::Trait {
                    display_type: None,
                    trait_type: "unbond_batch_id".to_string(),
                    value: "0".to_string(),
                },
                drop_staking_base::state::withdrawal_voucher::Trait {
                    display_type: None,
                    trait_type: "received_amount".to_string(),
                    value: "1000".to_string(),
                },
            ]),
        },
    );
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("withdrawal_voucher_contract").to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::withdrawal_voucher::ExecuteMsg::Mint {
                        token_id: "0_".to_string() + api.addr_make("some_sender").as_str() + "_1",
                        owner: api.addr_make("some_sender").to_string(),
                        token_uri: None,
                        extension,
                    }
                )
                .unwrap(),
                funds: vec![],
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api.addr_make("token_contract").to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Burn {}).unwrap(),
                funds: vec![Coin::new(1000u128, "ld_denom")],
            })))
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-unbond")
                    .add_attribute("action", "unbond")
            )
    );
    assert_eq!(
        unbond_batch,
        UnbondBatch {
            total_dasset_amount_to_withdraw: Uint128::from(1000u128),
            expected_native_asset_amount: Uint128::zero(),
            total_unbond_items: 1,
            status: UnbondBatchStatus::New,
            expected_release_time: 0,
            slashing_effect: None,
            unbonded_amount: None,
            withdrawn_amount: None,
            status_timestamps: get_default_unbond_batch_status_timestamps(),
        }
    );
}

#[test]
fn test_migrate_wrong_contract() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    cw2::set_contract_version(deps_mut.storage, "wrong_contract_name", "0.0.1").unwrap();

    let res = crate::contract::migrate(
        deps.as_mut(),
        mock_env(),
        drop_staking_base::msg::core::MigrateMsg {},
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

mod process_emergency_batch {
    use super::*;

    fn setup(
        status: UnbondBatchStatus,
    ) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery> {
        let mut deps = mock_dependencies(&[]);
        let owner_address = deps.api.addr_make("owner");

        {
            let deps_as_mut = deps.as_mut();
            cw_ownable::initialize_owner(
                deps_as_mut.storage,
                deps_as_mut.api,
                Some(owner_address.as_str()),
            )
            .unwrap();
        }
        {
            unbond_batches_map()
                .save(
                    deps.as_mut().storage,
                    2,
                    &UnbondBatch {
                        total_dasset_amount_to_withdraw: Uint128::new(100),
                        expected_native_asset_amount: Uint128::new(100),
                        expected_release_time: 200,
                        total_unbond_items: 0,
                        status,
                        slashing_effect: None,
                        unbonded_amount: None,
                        withdrawn_amount: None,
                        status_timestamps: get_default_unbond_batch_status_timestamps(),
                    },
                )
                .unwrap();
        }
        deps
    }

    #[test]
    fn unauthorized() {
        let mut deps = setup(UnbondBatchStatus::WithdrawnEmergency);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("stranger"), &[]),
            ExecuteMsg::ProcessEmergencyBatch {
                batch_id: 2,
                unbonded_amount: Uint128::new(100),
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
        );
    }

    #[test]
    fn not_in_withdrawn_emergency_state() {
        let mut deps = setup(UnbondBatchStatus::WithdrawingEmergency);
        let owner_address = deps.api.addr_make("owner");
        let err = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&owner_address, &[]),
            ExecuteMsg::ProcessEmergencyBatch {
                batch_id: 2,
                unbonded_amount: Uint128::new(100),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::BatchNotWithdrawnEmergency {});
    }

    #[test]
    fn unbonded_amount_is_zero() {
        let mut deps = setup(UnbondBatchStatus::WithdrawnEmergency);
        let owner_address = deps.api.addr_make("owner");
        let err = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&owner_address, &[]),
            ExecuteMsg::ProcessEmergencyBatch {
                batch_id: 2,
                unbonded_amount: Uint128::new(0),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::UnbondedAmountZero {});
    }

    #[test]
    fn unbonded_amount_too_high() {
        let mut deps = setup(UnbondBatchStatus::WithdrawnEmergency);
        let owner_address = deps.api.addr_make("owner");
        let err = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&owner_address, &[]),
            ExecuteMsg::ProcessEmergencyBatch {
                batch_id: 2,
                unbonded_amount: Uint128::new(200),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::UnbondedAmountTooHigh {});
    }

    #[test]
    fn no_slashing() {
        let mut deps = setup(UnbondBatchStatus::WithdrawnEmergency);
        let owner_address = deps.api.addr_make("owner");
        let shared_mock_env = mock_env();
        execute(
            deps.as_mut(),
            shared_mock_env.clone(),
            message_info(&owner_address, &[]),
            ExecuteMsg::ProcessEmergencyBatch {
                batch_id: 2,
                unbonded_amount: Uint128::new(100),
            },
        )
        .unwrap();

        let batch = unbond_batches_map().load(deps.as_mut().storage, 2).unwrap();
        assert_eq!(
            batch,
            UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::new(100),
                expected_native_asset_amount: Uint128::new(100),
                expected_release_time: 200,
                total_unbond_items: 0,
                status: UnbondBatchStatus::Withdrawn,
                slashing_effect: Some(Decimal::one()),
                unbonded_amount: Some(Uint128::new(100)),
                withdrawn_amount: None,
                status_timestamps: UnbondBatchStatusTimestamps {
                    new: 0,
                    unbond_requested: None,
                    unbond_failed: None,
                    unbonding: None,
                    withdrawing: None,
                    withdrawn: Some(shared_mock_env.block.time.seconds()),
                    withdrawing_emergency: None,
                    withdrawn_emergency: None,
                },
            }
        );
    }

    #[test]
    fn some_slashing() {
        let mut deps = setup(UnbondBatchStatus::WithdrawnEmergency);
        let owner_address = deps.api.addr_make("owner");
        let shared_mock_env = mock_env();
        execute(
            deps.as_mut(),
            shared_mock_env.clone(),
            message_info(&owner_address, &[]),
            ExecuteMsg::ProcessEmergencyBatch {
                batch_id: 2,
                unbonded_amount: Uint128::new(70),
            },
        )
        .unwrap();

        let batch = unbond_batches_map().load(deps.as_mut().storage, 2).unwrap();
        assert_eq!(
            batch,
            UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::new(100),
                expected_native_asset_amount: Uint128::new(100),
                expected_release_time: 200,
                total_unbond_items: 0,
                status: UnbondBatchStatus::Withdrawn,
                slashing_effect: Some(Decimal::from_ratio(70u128, 100u128)),
                unbonded_amount: Some(Uint128::new(70)),
                withdrawn_amount: None,
                status_timestamps: UnbondBatchStatusTimestamps {
                    new: 0,
                    unbond_requested: None,
                    unbond_failed: None,
                    unbonding: None,
                    withdrawing: None,
                    withdrawn: Some(shared_mock_env.block.time.seconds()),
                    withdrawing_emergency: None,
                    withdrawn_emergency: None,
                },
            }
        );
    }
}

mod bond_hooks {
    use super::*;
    use cosmwasm_std::{Binary, ReplyOn};
    use drop_helpers::testing::mock_state_query;
    use drop_staking_base::msg::core::{BondCallback, BondHook};
    use neutron_sdk::bindings::msg::NeutronMsg;

    #[test]
    fn set_bond_hooks_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        let owner_address = deps.api.addr_make("owner");

        {
            let deps_mut = deps.as_mut();
            cw_ownable::initialize_owner(
                deps_mut.storage,
                deps_mut.api,
                Some(owner_address.as_str()),
            )
            .unwrap();
        }

        let error = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("stranger"), &[]),
            ExecuteMsg::SetBondHooks { hooks: vec![] },
        )
        .unwrap_err();

        assert_eq!(
            error,
            ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
        );
    }

    #[test]
    fn set_bond_hooks() {
        let mut deps = mock_dependencies(&[]);
        let owner_address = deps.api.addr_make("owner");
        let val_ref_address = deps.api.addr_make("val_ref");

        {
            let deps_mut = deps.as_mut();
            cw_ownable::initialize_owner(
                deps_mut.storage,
                deps_mut.api,
                Some(owner_address.as_str()),
            )
            .unwrap();
        }

        let response = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&owner_address, &[]),
            ExecuteMsg::SetBondHooks {
                hooks: vec![val_ref_address.to_string()],
            },
        )
        .unwrap();

        assert_eq!(
            BOND_HOOKS.load(deps.as_ref().storage).unwrap(),
            vec![val_ref_address.clone()]
        );

        assert_eq!(
            response,
            Response::new().add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-set-bond-hooks")
                    .add_attribute("contract", val_ref_address)
            )
        );
    }

    #[test]
    fn set_bond_hooks_override() {
        let mut deps = mock_dependencies(&[]);
        let owner_address = deps.api.addr_make("owner");
        let validator_set_address = deps.api.addr_make("validator_set");

        {
            let deps_mut = deps.as_mut();
            cw_ownable::initialize_owner(
                deps_mut.storage,
                deps_mut.api,
                Some(owner_address.as_str()),
            )
            .unwrap();
        }

        BOND_HOOKS
            .save(deps.as_mut().storage, &vec![Addr::unchecked("val_ref")])
            .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&owner_address, &[]),
            ExecuteMsg::SetBondHooks {
                hooks: vec![validator_set_address.to_string()],
            },
        )
        .unwrap();

        assert_eq!(
            BOND_HOOKS.load(deps.as_ref().storage).unwrap(),
            vec![validator_set_address.clone()]
        );

        assert_eq!(
            response,
            Response::new().add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-set-bond-hooks")
                    .add_attribute("contract", validator_set_address)
            )
        );
    }

    #[test]
    fn set_bond_hooks_clear() {
        let mut deps = mock_dependencies(&[]);

        let owner_address = deps.api.addr_make("owner");

        {
            let deps_mut = deps.as_mut();
            cw_ownable::initialize_owner(
                deps_mut.storage,
                deps_mut.api,
                Some(owner_address.as_str()),
            )
            .unwrap();
        }

        BOND_HOOKS
            .save(deps.as_mut().storage, &vec![Addr::unchecked("val_ref")])
            .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&owner_address, &[]),
            ExecuteMsg::SetBondHooks { hooks: vec![] },
        )
        .unwrap();

        assert_eq!(
            BOND_HOOKS.load(deps.as_ref().storage).unwrap(),
            Vec::<Addr>::new()
        );

        assert_eq!(
            response,
            Response::new().add_event(Event::new(
                "crates.io:drop-staking__drop-core-execute-set-bond-hooks"
            ))
        );
    }

    #[test]
    fn query_bond_hooks() {
        let mut deps = mock_dependencies(&[]);

        BOND_HOOKS
            .save(deps.as_mut().storage, &vec![Addr::unchecked("val_ref")])
            .unwrap();

        assert_eq!(
            from_json::<Vec<String>>(
                &query(deps.as_ref(), mock_env(), QueryMsg::BondHooks {}).unwrap()
            )
            .unwrap(),
            vec![String::from("val_ref")]
        );
    }

    #[test]
    fn execute_bond_with_active_bond_hook_no_ref() {
        let mut deps = mock_dependencies(&[]);
        let api = deps.api;

        mock_state_query(&mut deps);
        BOND_PROVIDERS.init(deps.as_mut().storage).unwrap();

        deps.querier
            .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
                cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
            });

        deps.querier
            .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
                cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::from(1000u128)).unwrap())
            });

        FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
            .unwrap();
        BOND_PROVIDERS
            .add(
                deps.as_mut().storage,
                api.addr_make("native_provider_address"),
            )
            .unwrap();
        CONFIG
            .save(
                deps.as_mut().storage,
                &get_default_config(api, 1000, 100, 600),
            )
            .unwrap();
        LD_DENOM
            .save(deps.as_mut().storage, &"ld_denom".into())
            .unwrap();
        BOND_HOOKS
            .save(deps.as_mut().storage, &vec![api.addr_make("val_ref")])
            .unwrap();
        PAUSE
            .save(deps.as_mut().storage, &Pause::default())
            .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&api.addr_make("user"), &[Coin::new(1000u128, "base_denom")]),
            ExecuteMsg::Bond {
                receiver: None,
                r#ref: None,
            },
        )
        .unwrap();

        let bond_hook_messages = &response.messages[2..];
        assert_eq!(bond_hook_messages.len(), 1);
        assert_eq!(
            bond_hook_messages,
            vec![SubMsg {
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
                payload: Binary::default(),
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: api.addr_make("val_ref").to_string(),
                    funds: vec![],
                    msg: to_json_binary(&BondCallback::BondCallback(BondHook {
                        amount: Uint128::new(1000),
                        dasset_minted: Uint128::new(1000),
                        sender: api.addr_make("user"),
                        denom: String::from("base_denom"),
                        r#ref: None,
                    }))
                    .unwrap(),
                })
            }]
        );
    }

    #[test]
    fn execute_bond_with_active_bond_hook() {
        let mut deps = mock_dependencies(&[]);
        let api = deps.api;

        mock_state_query(&mut deps);
        BOND_PROVIDERS.init(deps.as_mut().storage).unwrap();

        deps.querier
            .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
                cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
            });

        deps.querier
            .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
                cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::from(1000u128)).unwrap())
            });

        FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
            .unwrap();

        BOND_PROVIDERS
            .add(
                deps.as_mut().storage,
                api.addr_make("native_provider_address"),
            )
            .unwrap();
        CONFIG
            .save(
                deps.as_mut().storage,
                &get_default_config(api, 1000, 100, 600),
            )
            .unwrap();
        LD_DENOM
            .save(deps.as_mut().storage, &"ld_denom".into())
            .unwrap();
        BOND_HOOKS
            .save(deps.as_mut().storage, &vec![api.addr_make("val_ref")])
            .unwrap();
        PAUSE
            .save(deps.as_mut().storage, &Pause::default())
            .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&api.addr_make("user"), &[Coin::new(1000u128, "base_denom")]),
            ExecuteMsg::Bond {
                receiver: None,
                r#ref: Some(api.addr_make("valoper").to_string()),
            },
        )
        .unwrap();

        let bond_hook_messages = &response.messages[2..];
        assert_eq!(bond_hook_messages.len(), 1);
        assert_eq!(
            bond_hook_messages,
            vec![SubMsg {
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
                payload: Binary::default(),
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: api.addr_make("val_ref").to_string(),
                    funds: vec![],
                    msg: to_json_binary(&BondCallback::BondCallback(BondHook {
                        amount: Uint128::new(1000),
                        dasset_minted: Uint128::new(1000),
                        sender: api.addr_make("user"),
                        denom: String::from("base_denom"),
                        r#ref: Some(api.addr_make("valoper").to_string()),
                    }))
                    .unwrap(),
                })
            }]
        );
    }

    #[test]
    fn execute_bond_with_active_bond_hooks() {
        let mut deps = mock_dependencies(&[]);
        let api = deps.api;
        BOND_PROVIDERS.init(deps.as_mut().storage).unwrap();

        deps.querier
            .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
                cosmwasm_std::ContractResult::Ok(to_json_binary(&true).unwrap())
            });
        mock_state_query(&mut deps);
        deps.querier
            .add_wasm_query_response(api.addr_make("native_provider_address").as_str(), |_| {
                cosmwasm_std::ContractResult::Ok(to_json_binary(&Uint128::from(1000u128)).unwrap())
            });

        let hooks = ["val_ref", "validator_set", "logger", "indexer"];

        FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
            .unwrap();

        BOND_PROVIDERS
            .add(
                deps.as_mut().storage,
                api.addr_make("native_provider_address"),
            )
            .unwrap();
        CONFIG
            .save(
                deps.as_mut().storage,
                &get_default_config(api, 1000, 100, 600),
            )
            .unwrap();
        LD_DENOM
            .save(deps.as_mut().storage, &"ld_denom".into())
            .unwrap();
        BOND_HOOKS
            .save(
                deps.as_mut().storage,
                &hooks.iter().map(|hook| api.addr_make(hook)).collect(),
            )
            .unwrap();
        PAUSE
            .save(deps.as_mut().storage, &Pause::default())
            .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&api.addr_make("user"), &[Coin::new(1000u128, "base_denom")]),
            ExecuteMsg::Bond {
                receiver: None,
                r#ref: Some(api.addr_make("valoper").to_string()),
            },
        )
        .unwrap();

        let bond_hook_messages = &response.messages[2..];
        assert_eq!(bond_hook_messages.len(), 4);
        assert_eq!(
            bond_hook_messages,
            hooks
                .iter()
                .map(|hook| SubMsg {
                    id: 0,
                    gas_limit: None,
                    reply_on: ReplyOn::Never,
                    payload: Binary::default(),
                    msg: CosmosMsg::<NeutronMsg>::Wasm(WasmMsg::Execute {
                        contract_addr: api.addr_make(hook).to_string(),
                        funds: vec![],
                        msg: to_json_binary(&BondCallback::BondCallback(BondHook {
                            amount: Uint128::new(1000),
                            dasset_minted: Uint128::new(1000),
                            sender: api.addr_make("user"),
                            denom: String::from("base_denom"),
                            r#ref: Some(api.addr_make("valoper").to_string()),
                        }))
                        .unwrap(),
                    })
                })
                .collect::<Vec<_>>()
        );
    }
}

mod pause {
    use super::*;

    #[test]
    fn pause_bond() {
        let mut deps = mock_dependencies(&[]);

        for pause in [
            Pause {
                bond: true,
                unbond: false,
                tick: false,
            },
            Pause {
                bond: true,
                unbond: true,
                tick: false,
            },
            Pause {
                bond: true,
                unbond: false,
                tick: true,
            },
            Pause {
                bond: true,
                unbond: true,
                tick: true,
            },
        ] {
            PAUSE.save(deps.as_mut().storage, &pause).unwrap();
            let error = execute(
                deps.as_mut(),
                mock_env(),
                message_info(&Addr::unchecked("someone"), &[]),
                ExecuteMsg::Bond {
                    receiver: None,
                    r#ref: None,
                },
            )
            .unwrap_err();
            assert_eq!(error, ContractError::PauseError(PauseError::Paused {}));
        }
    }

    #[test]
    fn pause_unbond() {
        let mut deps = mock_dependencies(&[]);

        for pause in [
            Pause {
                bond: false,
                unbond: true,
                tick: false,
            },
            Pause {
                bond: true,
                unbond: true,
                tick: false,
            },
            Pause {
                bond: false,
                unbond: true,
                tick: true,
            },
            Pause {
                bond: true,
                unbond: true,
                tick: true,
            },
        ] {
            PAUSE.save(deps.as_mut().storage, &pause).unwrap();
            let error = execute(
                deps.as_mut(),
                mock_env(),
                message_info(&Addr::unchecked("someone"), &[]),
                ExecuteMsg::Unbond {},
            )
            .unwrap_err();
            assert_eq!(error, ContractError::PauseError(PauseError::Paused {}));
        }
    }

    #[test]
    fn pause_tick() {
        let mut deps = mock_dependencies(&[]);

        for pause in [
            Pause {
                bond: false,
                unbond: false,
                tick: true,
            },
            Pause {
                bond: true,
                unbond: false,
                tick: true,
            },
            Pause {
                bond: false,
                unbond: true,
                tick: true,
            },
            Pause {
                bond: true,
                unbond: true,
                tick: true,
            },
        ] {
            PAUSE.save(deps.as_mut().storage, &pause).unwrap();
            let error = execute(
                deps.as_mut(),
                mock_env(),
                message_info(&Addr::unchecked("someone"), &[]),
                ExecuteMsg::Tick {},
            )
            .unwrap_err();
            assert_eq!(error, ContractError::PauseError(PauseError::Paused {}));
        }
    }
}
