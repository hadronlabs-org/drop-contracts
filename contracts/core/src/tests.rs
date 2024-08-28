use crate::contract::{
    check_denom::{DenomTrace, QueryDenomTraceResponse},
    execute, query,
};
use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Decimal256, Event, OwnedDeps,
    Response, SubMsg, Timestamp, Uint128, WasmMsg,
};
use drop_helpers::testing::{mock_dependencies, WasmMockQuerier};
use drop_puppeteer_base::{
    msg::TransferReadyBatchesMsg,
    state::{Delegations, DropDelegation, RedeemShareItem},
};
use drop_staking_base::state::core::{FAILED_BATCH_ID, LAST_STAKER_RESPONSE};
use drop_staking_base::{
    error::core::ContractError,
    msg::{
        core::{ExecuteMsg, FailedBatchResponse, InstantiateMsg},
        puppeteer::{BalancesResponse, DelegationsResponse},
        staker::QueryMsg as StakerQueryMsg,
        strategy::QueryMsg as StrategyQueryMsg,
    },
    state::core::{
        unbond_batches_map, Config, ConfigOptional, ContractState, UnbondBatch, UnbondBatchStatus,
        UnbondBatchStatusTimestamps, BONDED_AMOUNT, CONFIG, FSM, LAST_ICA_CHANGE_HEIGHT,
        LAST_IDLE_CALL, LAST_LSM_REDEEM, LAST_PUPPETEER_RESPONSE, LD_DENOM, LSM_SHARES_TO_REDEEM,
        PENDING_LSM_SHARES, TOTAL_LSM_SHARES, UNBOND_BATCH_ID,
    },
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v045::types::Balances,
    sudo::msg::RequestPacket,
};
use std::vec;

pub const MOCK_PUPPETEER_CONTRACT_ADDR: &str = "puppeteer_contract";
pub const MOCK_STRATEGY_CONTRACT_ADDR: &str = "strategy_contract";

fn get_default_config(
    idle_min_interval: u64,
    lsm_redeem_threshold: u64,
    lsm_redeem_maximum_interval: u64,
    unbonding_safe_period: u64,
    unbond_batch_switch_time: u64,
    lsm_min_bond_amount: Uint128,
) -> Config {
    Config {
        token_contract: Addr::unchecked("token_contract"),
        puppeteer_contract: Addr::unchecked(MOCK_PUPPETEER_CONTRACT_ADDR),
        strategy_contract: Addr::unchecked(MOCK_STRATEGY_CONTRACT_ADDR),
        withdrawal_voucher_contract: Addr::unchecked("withdrawal_voucher_contract"),
        withdrawal_manager_contract: Addr::unchecked("withdrawal_manager_contract"),
        validators_set_contract: Addr::unchecked("validators_set_contract"),
        staker_contract: Addr::unchecked("staker_contract"),
        base_denom: "base_denom".to_string(),
        remote_denom: "remote_denom".to_string(),
        idle_min_interval,
        unbonding_period: 60,
        unbonding_safe_period,
        unbond_batch_switch_time,
        pump_ica_address: Some("pump_address".to_string()),
        transfer_channel_id: "transfer_channel".to_string(),
        lsm_redeem_threshold,
        lsm_min_bond_amount,
        lsm_redeem_maximum_interval,
        bond_limit: None,
        emergency_address: None,
        min_stake_amount: Uint128::new(100),
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
    deps.querier
        .add_wasm_query_response("old_token_contract", |_| {
            to_json_binary(&drop_staking_base::msg::token::ConfigResponse {
                core_address: "core_contract".to_string(),
                denom: "ld_denom".to_string(),
            })
            .unwrap()
        });
    let env = mock_env();
    let info = mock_info("admin", &[]);
    let mut deps_mut = deps.as_mut();
    crate::contract::instantiate(
        deps_mut.branch(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            token_contract: "old_token_contract".to_string(),
            puppeteer_contract: "old_puppeteer_contract".to_string(),
            strategy_contract: "old_strategy_contract".to_string(),
            staker_contract: "old_staker_contract".to_string(),
            withdrawal_voucher_contract: "old_withdrawal_voucher_contract".to_string(),
            withdrawal_manager_contract: "old_withdrawal_manager_contract".to_string(),
            validators_set_contract: "old_validators_set_contract".to_string(),
            base_denom: "old_base_denom".to_string(),
            remote_denom: "old_remote_denom".to_string(),
            idle_min_interval: 12,
            unbonding_period: 20,
            unbonding_safe_period: 120,
            unbond_batch_switch_time: 2000,
            pump_ica_address: Some("old_pump_address".to_string()),
            transfer_channel_id: "old_transfer_channel".to_string(),
            lsm_redeem_max_interval: 20_000_000,
            lsm_redeem_threshold: 120u64,
            lsm_min_bond_amount: Uint128::new(12),
            bond_limit: Some(Uint128::new(12)),
            emergency_address: Some("old_emergency_address".to_string()),
            min_stake_amount: Uint128::new(1200),
            owner: "admin".to_string(),
            icq_update_delay: 5,
        },
    )
    .unwrap();
    assert_eq!(
        LD_DENOM.may_load(deps_mut.storage).unwrap(),
        Some("ld_denom".to_string())
    );

    let new_config = ConfigOptional {
        token_contract: Some("new_token_contract".to_string()),
        puppeteer_contract: Some("new_puppeteer_contract".to_string()),
        strategy_contract: Some("new_strategy_contract".to_string()),
        staker_contract: Some("new_staker_contract".to_string()),
        withdrawal_voucher_contract: Some("new_withdrawal_voucher_contract".to_string()),
        withdrawal_manager_contract: Some("new_withdrawal_manager_contract".to_string()),
        validators_set_contract: Some("new_validators_set_contract".to_string()),
        base_denom: Some("new_base_denom".to_string()),
        remote_denom: Some("new_remote_denom".to_string()),
        idle_min_interval: Some(2),
        unbonding_period: Some(120),
        unbonding_safe_period: Some(20),
        unbond_batch_switch_time: Some(12000),
        pump_ica_address: Some("new_pump_address".to_string()),
        transfer_channel_id: Some("new_transfer_channel".to_string()),
        rewards_receiver: Some("new_rewards_receiver".to_string()),
        lsm_redeem_threshold: Some(20u64),
        lsm_min_bond_amount: Some(Uint128::new(2)),
        lsm_redeem_maximum_interval: Some(20_000_000_000),
        bond_limit: Some(Uint128::new(2)),
        emergency_address: Some("new_emergency_address".to_string()),
        min_stake_amount: Some(Uint128::new(200)),
    };
    let expected_config = Config {
        token_contract: Addr::unchecked("new_token_contract"),
        puppeteer_contract: Addr::unchecked("new_puppeteer_contract"),
        staker_contract: Addr::unchecked("new_staker_contract"),
        strategy_contract: Addr::unchecked("new_strategy_contract"),
        withdrawal_voucher_contract: Addr::unchecked("new_withdrawal_voucher_contract"),
        withdrawal_manager_contract: Addr::unchecked("new_withdrawal_manager_contract"),
        validators_set_contract: Addr::unchecked("new_validators_set_contract"),
        base_denom: "new_base_denom".to_string(),
        remote_denom: "new_remote_denom".to_string(),
        idle_min_interval: 2,
        unbonding_period: 120,
        unbonding_safe_period: 20,
        unbond_batch_switch_time: 12000,
        pump_ica_address: Some("new_pump_address".to_string()),
        transfer_channel_id: "new_transfer_channel".to_string(),
        lsm_redeem_threshold: 20u64,
        lsm_min_bond_amount: Uint128::new(2),
        lsm_redeem_maximum_interval: 20_000_000_000,
        bond_limit: Some(Uint128::new(2)),
        emergency_address: Some("new_emergency_address".to_string()),
        min_stake_amount: Uint128::new(200),
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
fn test_update_withdrawn_amount() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 10, 10_000_000_000, 10, 6000, Uint128::one()),
        )
        .unwrap();

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
        mock_info("withdrawal_manager_contract", &[]),
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
        mock_info("withdrawal_manager_contract", &[]),
        ExecuteMsg::UpdateWithdrawnAmount {
            batch_id: 0,
            withdrawn_amount: Uint128::from(2002u128),
        },
    )
    .unwrap_err();
    assert_eq!(unbonding_err, ContractError::BatchNotWithdrawn {});
}

#[test]
fn test_execute_reset_bonded_amount() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("admin")).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::one())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        ExecuteMsg::ResetBondedAmount {},
    );
    assert_eq!(
        res,
        Ok(Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-core-execute-reset_bond_limit")
                .add_attributes(vec![("action", "reset_bond_limit"),])
        ))
    );
    let amount = BONDED_AMOUNT.load(deps.as_ref().storage).unwrap();
    assert_eq!(amount, Uint128::zero());
}

#[test]
fn test_execute_tick_idle_get_pending_lsm_shares_transfer() {
    let mut deps = mock_dependencies(&[]);
    for _ in 0..2 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
                            to_json_binary(&BalancesResponse {
                                balances: Balances { coins: vec![] },
                                remote_height: 10u64,
                                local_height: 10u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![DropDelegation {
                                        delegator: Addr::unchecked("delegator"),
                                        validator: "valoper1".to_string(),
                                        amount: Coin::new(1000, "remote_denom".to_string()),
                                        share_ratio: Decimal256::one(),
                                    }],
                                },
                                remote_height: 10u64,
                                local_height: 10u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
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
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 10, 10_000_000_000, 10, 6000, Uint128::one()),
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
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    LAST_LSM_REDEEM.save(deps.as_mut().storage, &0).unwrap();
    PENDING_LSM_SHARES
        .save(
            deps.as_mut().storage,
            "remote_denom".to_string(),
            &(
                "local_denom".to_string(),
                Uint128::from(100u128),
                Uint128::from(100u128),
            ),
        )
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[]),
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
                        ("knot", "041"),
                        ("knot", "042"),
                        ("knot", "043")
                    ]
                )
            )
            .add_submessages(vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
                        reason: drop_puppeteer_base::msg::IBCTransferReason::LSMShare,
                        reply_to: "cosmos2contract".to_string(),
                    }
                )
                .unwrap(),
                funds: vec![Coin::new(100, "remote_denom")],
            }))])
    );
}

#[test]
fn test_idle_tick_pending_lsm_redeem() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 10, 6000, Uint128::one()),
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
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    LAST_LSM_REDEEM.save(deps.as_mut().storage, &0).unwrap();
    LSM_SHARES_TO_REDEEM
        .save(
            deps.as_mut().storage,
            "remote_denom_share1".to_string(),
            &(
                "local_denom_1".to_string(),
                Uint128::from(100u128),
                Uint128::from(100u128),
            ),
        )
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(99);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("admin", &[]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());

    LSM_SHARES_TO_REDEEM
        .save(
            deps.as_mut().storage,
            "remote_denom_share2".to_string(),
            &(
                "local_denom_2".to_string(),
                Uint128::from(100u128),
                Uint128::from(100u128),
            ),
        )
        .unwrap();
    LSM_SHARES_TO_REDEEM
        .save(
            deps.as_mut().storage,
            "remote_denom_share3".to_string(),
            &(
                "local_denom_3".to_string(),
                Uint128::from(100u128),
                Uint128::from(100u128),
            ),
        )
        .unwrap();

    for _ in 0..3 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
                            to_json_binary(&BalancesResponse {
                                balances: Balances { coins: vec![] },
                                remote_height: 10u64,
                                local_height: 10u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![],
                                },
                                remote_height: 10u64,
                                local_height: 10u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
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
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[]),
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
                        ("knot", "037"),
                        ("knot", "038")
                    ]
                )
            )
            .add_submessages(vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
                        items: vec![
                            RedeemShareItem {
                                amount: Uint128::from(100u128),
                                remote_denom: "local_denom_1".to_string(),
                                local_denom: "remote_denom_share1".to_string()
                            },
                            RedeemShareItem {
                                amount: Uint128::from(100u128),
                                remote_denom: "local_denom_2".to_string(),
                                local_denom: "remote_denom_share2".to_string()
                            },
                            RedeemShareItem {
                                amount: Uint128::from(100u128),
                                remote_denom: "local_denom_3".to_string(),
                                local_denom: "remote_denom_share3".to_string()
                            }
                        ],
                        reply_to: "cosmos2contract".to_string()
                    }
                )
                .unwrap(),
                funds: vec![],
            }))])
    );
}

#[test]
fn test_tick_idle_unbonding_close() {
    let mut deps = mock_dependencies(&[]);
    for _ in 0..3 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
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
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![DropDelegation {
                                        delegator: Addr::unchecked("ica_address"),
                                        validator: "valoper_address".to_string(),
                                        amount: Coin {
                                            denom: "remote_denom".to_string(),
                                            amount: Uint128::new(100_000),
                                        },
                                        share_ratio: Decimal256::one(),
                                    }],
                                },
                                remote_height: 10u64,
                                local_height: 10u64,
                                timestamp: Timestamp::from_seconds(0),
                            })
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&vec![
                drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper_address".to_string(),
                    weight: 1,
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
            .unwrap()
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 6000, Uint128::one()),
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
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::from(1000u128))
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
                expected_release_time: 10001,
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(10000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::UnbondingTimeIsClose {}));
}

#[test]
fn test_tick_idle_claim_wo_unbond() {
    let mut deps = mock_dependencies(&[]);
    for _ in 0..5 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
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
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![DropDelegation {
                                        delegator: Addr::unchecked("ica_address"),
                                        validator: "valoper_address".to_string(),
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
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&vec![
                drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper_address".to_string(),
                    weight: 1,
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
            .unwrap()
        });
    let mut config = get_default_config(1000, 3, 100, 100, 6000, Uint128::one());
    config.lsm_redeem_maximum_interval = 100;
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
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::from(1000u128))
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(10000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
                        ("validators_to_claim", "valoper_address"),
                        ("knot", "011"),
                        ("knot", "012"),
                        ("state", "claiming"),
                    ]
                )
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                    validators: vec!["valoper_address".to_string()], 
                    transfer: None,
                    reply_to: "cosmos2contract".to_string() 
                }).unwrap(),
                funds: vec![Coin::new(1000, "untrn")],
            })))
    );
}

#[test]
fn test_tick_idle_claim_with_unbond_transfer() {
    let mut deps = mock_dependencies(&[]);
    for _ in 0..5 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
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
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![DropDelegation {
                                        delegator: Addr::unchecked("ica_address"),
                                        validator: "valoper_address".to_string(),
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
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&vec![
                drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper_address".to_string(),
                    weight: 1,
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 6000, Uint128::one()),
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
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::from(1000u128))
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
            ("validators_to_claim",  "valoper_address"), 
            ("knot", "011"),
            ("knot", "012"),
            ("state",  "claiming"),
        ]))
        .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "puppeteer_contract".to_string(), 
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                validators: vec!["valoper_address".to_string()], 
                transfer: Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg{ batch_ids: vec![0u128], emergency: false, amount: Uint128::from(200u128), recipient: "pump_address".to_string() }), 
                reply_to: "cosmos2contract".to_string()                 
            }).unwrap(), funds: vec![Coin::new(1000, "untrn")] })))
    );
}

#[test]
fn test_tick_idle_staking_bond() {
    let mut deps = mock_dependencies(&[Coin::new(1000u128, "base_denom")]);
    for _ in 0..5 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
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
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![],
                                },
                                remote_height: 12344u64,
                                local_height: 12344u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::new(0),
                expected_native_asset_amount: Uint128::new(0),
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
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&vec![
                drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper_address".to_string(),
                    weight: 1,
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
            .unwrap()
        });
    for _ in 0..2 {
        deps.querier
            .add_wasm_query_response("staker_contract", |_| {
                to_json_binary(&Uint128::from(100000u128)).unwrap()
            });
    }
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: drop_staking_base::msg::strategy::QueryMsg = from_json(msg).unwrap();
            match q {
                drop_staking_base::msg::strategy::QueryMsg::CalcDeposit { deposit } => {
                    to_json_binary(&vec![("valoper_address".to_string(), deposit)]).unwrap()
                }
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
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
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::from(1000u128))
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
                        ("validators_to_claim", "empty"),
                        ("knot", "015"),
                        ("knot", "016"),
                        ("knot", "017"),
                        ("state", "staking_bond"),
                    ]
                )
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "staker_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::staker::ExecuteMsg::Stake {
                    items: vec![("valoper_address".to_string(), Uint128::from(100000u128))]
                })
                .unwrap(),
                funds: vec![Coin::new(1000, "untrn")],
            })))
    );
}

#[test]
fn test_tick_idle_unbonding() {
    let mut deps = mock_dependencies(&[]);
    for _ in 0..6 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
                            to_json_binary(&BalancesResponse {
                                balances: Balances { coins: vec![] },
                                remote_height: 10u64,
                                local_height: 10u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![],
                                },
                                remote_height: 12344u64,
                                local_height: 12344u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    for _ in 0..3 {
        deps.querier
            .add_wasm_query_response("staker_contract", |_| {
                to_json_binary(&Uint128::zero()).unwrap()
            });
    }
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&vec![
                drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper_address".to_string(),
                    weight: 1,
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |_| {
            to_json_binary(&vec![(
                "valoper_address".to_string(),
                Uint128::from(1000u128),
            )])
            .unwrap()
        });

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 6000, Uint128::one()),
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
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::from(1000u128))
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
                        ("validators_to_claim", "empty"),
                        ("knot", "015"),
                        ("knot", "017"),
                        ("knot", "024"),
                        ("knot", "026"),
                        ("knot", "027"),
                        ("exchange_rate", "1"),
                        ("knot", "045"),
                        ("knot", "049"),
                        ("knot", "046"),
                        ("knot", "028"),
                        ("knot", "029"),
                        ("state", "unbonding"),
                    ]
                )
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
                    batch_id: 0u128,
                    items: vec![("valoper_address".to_string(), Uint128::from(1000u128))],
                    reply_to: "cosmos2contract".to_string()
                })
                .unwrap(),
                funds: vec![Coin::new(1000, "untrn")],
            })))
    );
}

#[test]
fn test_tick_idle_unbonding_failed() {
    let mut deps = mock_dependencies(&[]);
    for _ in 0..6 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |msg| {
                let msg = from_json::<
                    drop_puppeteer_base::msg::QueryMsg<
                        drop_staking_base::msg::puppeteer::QueryExtMsg,
                    >,
                >(msg)
                .unwrap();
                match msg {
                    drop_puppeteer_base::msg::QueryMsg::Extension { msg } => match msg {
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {} => {
                            to_json_binary(&BalancesResponse {
                                balances: Balances { coins: vec![] },
                                remote_height: 10u64,
                                local_height: 10u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                            to_json_binary(&DelegationsResponse {
                                delegations: Delegations {
                                    delegations: vec![],
                                },
                                remote_height: 12344u64,
                                local_height: 12344u64,
                                timestamp: Timestamp::from_seconds(90001),
                            })
                            .unwrap()
                        }
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            });
    }
    for _ in 0..3 {
        deps.querier
            .add_wasm_query_response("staker_contract", |_| {
                to_json_binary(&Uint128::zero()).unwrap()
            });
    }
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&vec![
                drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper_address".to_string(),
                    weight: 1,
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |_| {
            to_json_binary(&vec![(
                "valoper_address".to_string(),
                Uint128::from(1000u128),
            )])
            .unwrap()
        });

    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 6000, Uint128::one()),
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
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::from(1000u128))
        .unwrap();
    FAILED_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &1).unwrap();
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            0,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(1000u128),
                expected_native_asset_amount: Uint128::from(1000u128),
                total_unbond_items: 1,
                status: UnbondBatchStatus::UnbondFailed,
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
    unbond_batches_map()
        .save(
            deps.as_mut().storage,
            1,
            &UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(1000u128),
                expected_native_asset_amount: Uint128::from(123123u128),
                total_unbond_items: 123,
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
                        ("validators_to_claim", "empty"),
                        ("knot", "015"),
                        ("knot", "017"),
                        ("knot", "024"),
                        ("knot", "025"),
                        ("knot", "027"),
                        ("exchange_rate", "1"),
                        ("knot", "045"),
                        ("knot", "049"),
                        ("knot", "028"),
                        ("knot", "029"),
                        ("state", "unbonding")
                    ]
                )
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
                    batch_id: 0u128,
                    items: vec![("valoper_address".to_string(), Uint128::from(1000u128))],
                    reply_to: "cosmos2contract".to_string()
                })
                .unwrap(),
                funds: vec![Coin::new(1000, "untrn")],
            })))
    );
    let current_batch_id = UNBOND_BATCH_ID.load(deps.as_ref().storage).unwrap();
    assert_eq!(current_batch_id, 1);
    let batch = unbond_batches_map().load(deps.as_ref().storage, 1).unwrap();
    assert_eq!(batch.status, UnbondBatchStatus::New);
    assert_eq!(batch.total_unbond_items, 123);
    assert_eq!(
        batch.expected_native_asset_amount,
        Uint128::from(123123u128)
    );
}

#[test]
fn test_tick_no_puppeteer_response() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::PuppeteerResponseIsNotReceived {}));
}

#[test]
fn test_tick_claiming_error_wo_transfer() {
    // no unbonded batch, no pending transfer for stake, some balance in ICA to stake
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("staker_contract", |_| {
            to_json_binary(&Uint128::zero()).unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcDeposit { deposit } => {
                    to_json_binary(&vec![("valoper_address".to_string(), deposit)]).unwrap()
                }
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::msg::ResponseHookMsg::Error(
                drop_puppeteer_base::msg::ResponseHookErrorMsg {
                    details: "Some error".to_string(),
                    request_id: 0u64,
                    request: null_request_packet(),
                    transaction:
                        drop_puppeteer_base::msg::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec!["valoper_address".to_string()],
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
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
                    ("error_on_claiming", "ResponseHookErrorMsg { request_id: 0, transaction: ClaimRewardsAndOptionalyTransfer { interchain_account_id: \"ica\", validators: [\"valoper_address\"], denom: \"remote_denom\", transfer: None }, request: RequestPacket { sequence: None, source_port: None, source_channel: None, destination_port: None, destination_channel: None, data: None, timeout_height: None, timeout_timestamp: None }, details: \"Some error\" }"),
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
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("staker_contract", |_| {
            to_json_binary(&Uint128::zero()).unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcDeposit { deposit } => {
                    to_json_binary(&vec![("valoper_address".to_string(), deposit)]).unwrap()
                }
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
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
            &drop_puppeteer_base::msg::ResponseHookMsg::Error(
                drop_puppeteer_base::msg::ResponseHookErrorMsg {
                    details: "Some error".to_string(),
                    request_id: 0u64,
                    request: null_request_packet(),
                    transaction:
                        drop_puppeteer_base::msg::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec!["valoper_address".to_string()],
                            denom: "remote_denom".to_string(),
                            transfer: Some(TransferReadyBatchesMsg {
                                batch_ids: vec![0u128],
                                emergency: false,
                                amount: Uint128::new(123123u128),
                                recipient: "recipient".to_string(),
                            }),
                        },
                },
            ),
        )
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
                    ("error_on_claiming", "ResponseHookErrorMsg { request_id: 0, transaction: ClaimRewardsAndOptionalyTransfer { interchain_account_id: \"ica\", validators: [\"valoper_address\"], denom: \"remote_denom\", transfer: Some(TransferReadyBatchesMsg { batch_ids: [0], emergency: false, amount: Uint128(123123), recipient: \"recipient\" }) }, request: RequestPacket { sequence: None, source_port: None, source_channel: None, destination_port: None, destination_channel: None, data: None, timeout_height: None, timeout_timestamp: None }, details: \"Some error\" }"),
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
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("staker_contract", |_| {
            to_json_binary(&Uint128::zero()).unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcWithdraw { withdraw } => {
                    to_json_binary(&vec![("valoper_address".to_string(), withdraw)]).unwrap()
                }
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::msg::ResponseHookMsg::Success(
                drop_puppeteer_base::msg::ResponseHookSuccessMsg {
                    request_id: 0u64,
                    request: null_request_packet(),
                    local_height: 9u64,
                    remote_height: 9u64,
                    transaction:
                        drop_puppeteer_base::msg::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec!["valoper_address".to_string()],
                            denom: "remote_denom".to_string(),
                            transfer: None,
                        },
                    answers: vec![],
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
                    items: vec![("valoper_address".to_string(), Uint128::from(1000u128))],
                    batch_id: 0u128,
                    reply_to: "cosmos2contract".to_string()
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
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("staker_contract", |_| {
            to_json_binary(&Uint128::zero()).unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcWithdraw { withdraw } => {
                    to_json_binary(&vec![("valoper_address".to_string(), withdraw)]).unwrap()
                }
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 60000, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    FSM.go_to(deps.as_mut().storage, ContractState::Claiming)
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::msg::ResponseHookMsg::Success(
                drop_puppeteer_base::msg::ResponseHookSuccessMsg {
                    request_id: 0u64,
                    request: null_request_packet(),
                    local_height: 9u64,
                    remote_height: 9u64,
                    transaction:
                        drop_puppeteer_base::msg::Transaction::ClaimRewardsAndOptionalyTransfer {
                            interchain_account_id: "ica".to_string(),
                            validators: vec!["valoper_address".to_string()],
                            denom: "remote_denom".to_string(),
                            transfer: None,
                        },
                    answers: vec![],
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);

    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
fn test_execute_tick_transfering_no_puppeteer_response() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::StakingBond)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::StakerResponseIsNotReceived {}));
}

#[test]
fn test_execute_tick_guard_balance_outdated() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &11)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &11)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 12u64,
                local_height: 12u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
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
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Unbonding)
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::PuppeteerResponseIsNotReceived {}));
}

#[test]
fn test_tick_staking_to_unbonding() {
    // we have no balance on ICA to stake, but we have unbond batch to switch
    let mut deps = mock_dependencies(&[]);

    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::msg::ResponseHookMsg::Success(
                drop_puppeteer_base::msg::ResponseHookSuccessMsg {
                    request_id: 0u64,
                    request: null_request_packet(),
                    local_height: 9u64,
                    remote_height: 9u64,
                    transaction: drop_puppeteer_base::msg::Transaction::IBCTransfer {
                        denom: "remote_denom".to_string(),
                        amount: 0u128,
                        recipient: "ICA".to_string(),
                        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
                    },
                    answers: vec![],
                },
            ),
        )
        .unwrap();
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &9u64)
        .unwrap();
    LAST_STAKER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::msg::staker::ResponseHookMsg::Success(
                drop_staking_base::msg::staker::ResponseHookSuccessMsg {
                    request_id: 1,
                    request: null_request_packet(),
                    transaction: drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(1000u128),
                    },
                    local_height: 1,
                    remote_height: 1,
                },
            ),
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcWithdraw { withdraw } => {
                    to_json_binary(&vec![("valoper_address".to_string(), withdraw)]).unwrap()
                }
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 1000, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::StakingBond)
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(2000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-tick_staking_bond")
                    .add_attributes(vec![
                        ("action", "tick_staking_bond"),
                        ("knot", "017"),
                        ("knot", "024"),
                        ("knot", "026"),
                        ("knot", "027"),
                        ("exchange_rate", "1"),
                        ("knot", "045"),
                        ("knot", "049"),
                        ("knot", "046"),
                        ("knot", "028"),
                        ("knot", "029"),
                        ("state", "unbonding"),
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
                    items: vec![("valoper_address".to_string(), Uint128::from(1000u128))],
                    batch_id: 0u128,
                    reply_to: "cosmos2contract".to_string()
                })
                .unwrap(),
                funds: vec![Coin::new(1000u128, "untrn")],
            })))
    );
}

#[test]
fn test_tick_staking_to_idle() {
    // we have no balance on ICA to stake, and the unbond batch is not ready to switch
    let mut deps = mock_dependencies(&[]);
    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    LAST_PUPPETEER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::msg::ResponseHookMsg::Success(
                drop_puppeteer_base::msg::ResponseHookSuccessMsg {
                    request_id: 0u64,
                    request: null_request_packet(),
                    local_height: 9u64,
                    remote_height: 9u64,
                    transaction: drop_puppeteer_base::msg::Transaction::IBCTransfer {
                        denom: "remote_denom".to_string(),
                        amount: 0u128,
                        recipient: "ICA".to_string(),
                        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
                    },
                    answers: vec![],
                },
            ),
        )
        .unwrap();
    LAST_STAKER_RESPONSE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::msg::staker::ResponseHookMsg::Success(
                drop_staking_base::msg::staker::ResponseHookSuccessMsg {
                    request_id: 1,
                    request: null_request_packet(),
                    transaction: drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(1000u128),
                    },
                    local_height: 1,
                    remote_height: 1,
                },
            ),
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
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
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StrategyQueryMsg = from_json(msg).unwrap();
            match q {
                StrategyQueryMsg::CalcWithdraw { withdraw } => {
                    to_json_binary(&vec![("valoper_address".to_string(), withdraw)]).unwrap()
                }
                _ => unimplemented!(),
            }
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 10000, Uint128::one()),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::StakingBond)
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(2000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[Coin::new(1000, "untrn")]),
        ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-core-execute-tick_staking_bond")
                .add_attributes(vec![
                    ("action", "tick_staking_bond"),
                    ("knot", "017"),
                    ("knot", "024"),
                    ("knot", "026"),
                    ("knot", "027"),
                    ("knot", "000"),
                    ("state", "idle"),
                ])
        )
    );
}

#[test]
fn test_execute_tick_unbonding_no_puppeteer_response() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();

    LAST_ICA_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&BalancesResponse {
                balances: Balances { coins: vec![] },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Unbonding)
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[Coin::new(1000, "untrn")]),
        ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::PuppeteerResponseIsNotReceived {}));
}

#[test]
fn test_bond_wo_receiver() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![DropDelegation {
                        delegator: Addr::unchecked("delegator"),
                        validator: "valoper1".to_string(),
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
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
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
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    TOTAL_LSM_SHARES
        .save(deps.as_mut().storage, &0u128)
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some", &[Coin::new(1000, "base_denom")]),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    )
    .unwrap();
    let bonded_amount = BONDED_AMOUNT.load(deps.as_ref().storage).unwrap();
    assert_eq!(bonded_amount, Uint128::from(1000u128));
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-bond")
                    .add_attribute("action", "bond")
                    .add_attribute("exchange_rate", "1")
                    .add_attribute("issue_amount", "1000")
                    .add_attribute("receiver", "some")
            )
            .add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "staker_contract".to_string(),
                amount: vec![Coin::new(1000, "base_denom")]
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Mint {
                    amount: Uint128::from(1000u128),
                    receiver: "some".to_string()
                })
                .unwrap(),
                funds: vec![],
            })))
    );
}

#[test]
fn test_bond_with_receiver() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![DropDelegation {
                        delegator: Addr::unchecked("delegator"),
                        validator: "valoper1".to_string(),
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
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
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
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    TOTAL_LSM_SHARES
        .save(deps.as_mut().storage, &0u128)
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some", &[Coin::new(1000, "base_denom")]),
        ExecuteMsg::Bond {
            receiver: Some("receiver".to_string()),
            r#ref: Some("ref".to_string()),
        },
    )
    .unwrap();
    let bonded_amount = BONDED_AMOUNT.load(deps.as_ref().storage).unwrap();
    assert_eq!(bonded_amount, Uint128::from(1000u128));
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-bond")
                    .add_attribute("action", "bond")
                    .add_attribute("exchange_rate", "1")
                    .add_attribute("issue_amount", "1000")
                    .add_attribute("receiver", "receiver")
                    .add_attribute("ref", "ref")
            )
            .add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "staker_contract".to_string(),
                amount: vec![Coin::new(1000, "base_denom")]
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Mint {
                    amount: Uint128::from(1000u128),
                    receiver: "receiver".to_string()
                })
                .unwrap(),
                funds: vec![],
            })))
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
fn test_bond_lsm_share_wrong_channel() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_stargate_query_response(
        "/ibc.applications.transfer.v1.Query/DenomTrace",
        |_data| {
            to_json_binary(&QueryDenomTraceResponse {
                denom_trace: DenomTrace {
                    path: "transfer/wrong_channel".to_string(),
                    base_denom: "valoper1/1".to_string(),
                },
            })
            .unwrap()
        },
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::new(100)),
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some", &[Coin::new(1000, "lsm_share")]),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::InvalidDenom {}));
}

#[test]
fn test_bond_lsm_share_increase_exchange_rate() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "ld_denom".to_string(),
        amount: Uint128::new(1001),
    }]);
    deps.querier.add_stargate_query_response(
        "/ibc.applications.transfer.v1.Query/DenomTrace",
        |_data| {
            to_json_binary(&QueryDenomTraceResponse {
                denom_trace: DenomTrace {
                    path: "transfer/transfer_channel".to_string(),
                    base_denom: "valoper1/1".to_string(),
                },
            })
            .unwrap()
        },
    );
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper1".to_string(),
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
        });
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![DropDelegation {
                        delegator: Addr::unchecked("delegator"),
                        validator: "valoper1".to_string(),
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
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![DropDelegation {
                        delegator: Addr::unchecked("delegator"),
                        validator: "valoper1".to_string(),
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
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    TOTAL_LSM_SHARES
        .save(deps.as_mut().storage, &0u128)
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::new(1)),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    UNBOND_BATCH_ID.save(&mut deps.storage, &0).unwrap();
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
        mock_info("some", &[Coin::new(100500, "lsm_share")]),
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
fn test_bond_lsm_share_wrong_validator() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_stargate_query_response(
        "/ibc.applications.transfer.v1.Query/DenomTrace",
        |_data| {
            to_json_binary(&QueryDenomTraceResponse {
                denom_trace: DenomTrace {
                    path: "transfer/transfer_channel".to_string(),
                    base_denom: "outside_valoper1/1".to_string(),
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
            if let drop_staking_base::msg::validatorset::QueryMsg::Validator { valoper } = request {
                assert_eq!(valoper, "outside_valoper1".to_string());
                query_called_cb.replace(true);
                to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                    validator: None,
                })
                .unwrap()
            } else {
                unimplemented!()
            }
        });
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::new(100)),
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some", &[Coin::new(1000, "lsm_share")]),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    );
    assert!(res.is_err());
    assert_eq!(res, Err(ContractError::InvalidDenom {}));
    assert!(*query_called.borrow());
}

#[test]
fn test_bond_lsm_share_ok() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_stargate_query_response(
        "/ibc.applications.transfer.v1.Query/DenomTrace",
        |_data| {
            to_json_binary(&QueryDenomTraceResponse {
                denom_trace: DenomTrace {
                    path: "transfer/transfer_channel".to_string(),
                    base_denom: "valoper1/1".to_string(),
                },
            })
            .unwrap()
        },
    );
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper1".to_string(),
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
        });
    for _ in 0..2 {
        deps.querier
            .add_wasm_query_response("puppeteer_contract", |_| {
                to_json_binary(&DelegationsResponse {
                    delegations: Delegations {
                        delegations: vec![DropDelegation {
                            delegator: Addr::unchecked("delegator"),
                            validator: "valoper1".to_string(),
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
    deps.querier
        .add_wasm_query_response("staker_contract", |data| {
            let req: StakerQueryMsg = from_json(data).unwrap();
            match req {
                StakerQueryMsg::AllBalance {} => to_json_binary(&Uint128::new(1)).unwrap(),
                _ => unimplemented!(),
            }
        });
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    TOTAL_LSM_SHARES
        .save(deps.as_mut().storage, &0u128)
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::new(100)),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    UNBOND_BATCH_ID.save(deps.as_mut().storage, &0).unwrap();
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
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some", &[Coin::new(1000, "lsm_share")]),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    )
    .unwrap();
    let bonded_amount = BONDED_AMOUNT.load(deps.as_ref().storage).unwrap();
    let total_lsm_shares = TOTAL_LSM_SHARES.load(deps.as_ref().storage).unwrap();
    assert_eq!(bonded_amount, Uint128::from(1000u128));
    assert_eq!(total_lsm_shares, 1000u128);
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-bond")
                    .add_attribute("action", "bond")
                    .add_attribute("exchange_rate", "1")
                    .add_attribute("issue_amount", "1000")
                    .add_attribute("receiver", "some")
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Mint {
                    amount: Uint128::from(1000u128),
                    receiver: "some".to_string()
                })
                .unwrap(),
                funds: vec![],
            })))
    );
}

#[test]
fn test_bond_lsm_share_ok_with_low_ratio() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_stargate_query_response(
        "/ibc.applications.transfer.v1.Query/DenomTrace",
        |_data| {
            to_json_binary(&QueryDenomTraceResponse {
                denom_trace: DenomTrace {
                    path: "transfer/transfer_channel".to_string(),
                    base_denom: "valoper1/1".to_string(),
                },
            })
            .unwrap()
        },
    );
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper1".to_string(),
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
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![DropDelegation {
                        delegator: Addr::unchecked("delegator"),
                        validator: "valoper1".to_string(),
                        amount: Coin::new(1000, "remote_denom".to_string()),
                        share_ratio: Decimal256::from_ratio(1u32, 2u32),
                    }],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    TOTAL_LSM_SHARES
        .save(deps.as_mut().storage, &0u128)
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::new(100)),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some", &[Coin::new(1000, "lsm_share")]),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    )
    .unwrap();
    let bonded_amount = BONDED_AMOUNT.load(deps.as_ref().storage).unwrap();
    let total_lsm_shares = TOTAL_LSM_SHARES.load(deps.as_ref().storage).unwrap();
    let pending_lsm_shares = PENDING_LSM_SHARES
        .load(deps.as_ref().storage, "lsm_share".to_string())
        .unwrap();
    assert_eq!(
        pending_lsm_shares,
        (
            "valoper1/1".to_string(),
            Uint128::from(1000u128),
            Uint128::from(500u128)
        )
    );
    assert_eq!(bonded_amount, Uint128::from(500u128));
    assert_eq!(total_lsm_shares, 500u128);
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-bond")
                    .add_attribute("action", "bond")
                    .add_attribute("exchange_rate", "1")
                    .add_attribute("issue_amount", "500")
                    .add_attribute("receiver", "some")
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Mint {
                    amount: Uint128::from(500u128),
                    receiver: "some".to_string()
                })
                .unwrap(),
                funds: vec![],
            })))
    );
}

#[test]
fn test_bond_lsm_share_ok_with_low_ratio_pending_already_there() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_stargate_query_response(
        "/ibc.applications.transfer.v1.Query/DenomTrace",
        |_data| {
            to_json_binary(&QueryDenomTraceResponse {
                denom_trace: DenomTrace {
                    path: "transfer/transfer_channel".to_string(),
                    base_denom: "valoper1/1".to_string(),
                },
            })
            .unwrap()
        },
    );
    deps.querier
        .add_wasm_query_response("validators_set_contract", |_| {
            to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
                validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: "valoper1".to_string(),
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
        });
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![DropDelegation {
                        delegator: Addr::unchecked("delegator"),
                        validator: "valoper1".to_string(),
                        amount: Coin::new(1000, "remote_denom".to_string()),
                        share_ratio: Decimal256::from_ratio(1u32, 2u32),
                    }],
                },
                remote_height: 10u64,
                local_height: 10u64,
                timestamp: Timestamp::from_seconds(90001),
            })
            .unwrap()
        });
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    TOTAL_LSM_SHARES
        .save(deps.as_mut().storage, &0u128)
        .unwrap();
    PENDING_LSM_SHARES
        .save(
            deps.as_mut().storage,
            "lsm_share".to_string(),
            &(
                "valoper1/1".to_string(),
                Uint128::from(10000u128),
                Uint128::from(5000u128),
            ),
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(1000, 3, 100, 100, 600, Uint128::new(100)),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some", &[Coin::new(1000, "lsm_share")]),
        ExecuteMsg::Bond {
            receiver: None,
            r#ref: None,
        },
    )
    .unwrap();
    let bonded_amount = BONDED_AMOUNT.load(deps.as_ref().storage).unwrap();
    let total_lsm_shares = TOTAL_LSM_SHARES.load(deps.as_ref().storage).unwrap();
    let pending_lsm_shares = PENDING_LSM_SHARES
        .load(deps.as_ref().storage, "lsm_share".to_string())
        .unwrap();
    assert_eq!(
        pending_lsm_shares,
        (
            "valoper1/1".to_string(),
            Uint128::from(11000u128),
            Uint128::from(5500u128)
        )
    );
    assert_eq!(bonded_amount, Uint128::from(500u128));
    assert_eq!(total_lsm_shares, 500u128);
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-bond")
                    .add_attribute("action", "bond")
                    .add_attribute("exchange_rate", "1")
                    .add_attribute("issue_amount", "500")
                    .add_attribute("receiver", "some")
            )
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::token::ExecuteMsg::Mint {
                    amount: Uint128::from(500u128),
                    receiver: "some".to_string()
                })
                .unwrap(),
                funds: vec![],
            })))
    );
}

#[test]
fn test_unbond() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1000);
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::from(1000u128))
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
            &get_default_config(1000, 3, 100, 100, 600, Uint128::one()),
        )
        .unwrap();
    LD_DENOM
        .save(deps.as_mut().storage, &"ld_denom".into())
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("some_sender", &[Coin::new(1000, "ld_denom")]),
        ExecuteMsg::Unbond {},
    )
    .unwrap();
    let unbond_batch = unbond_batches_map().load(deps.as_ref().storage, 0).unwrap();
    let extension = Some(drop_staking_base::state::withdrawal_voucher::Metadata {
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
    });
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "withdrawal_voucher_contract".to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::withdrawal_voucher::ExecuteMsg::Mint {
                        token_id: "0_some_sender_1".to_string(),
                        owner: "some_sender".to_string(),
                        token_uri: None,
                        extension,
                    }
                )
                .unwrap(),
                funds: vec![],
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "token_contract".to_string(),
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
    let bonded_amount = BONDED_AMOUNT.load(deps.as_ref().storage).unwrap();
    assert_eq!(bonded_amount, Uint128::zero());
}

fn null_request_packet() -> RequestPacket {
    RequestPacket {
        sequence: None,
        source_port: None,
        source_channel: None,
        destination_port: None,
        destination_channel: None,
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    }
}

mod process_emergency_batch {
    use super::*;

    fn setup(
        status: UnbondBatchStatus,
    ) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery> {
        let mut deps = mock_dependencies(&[]);
        {
            let deps_as_mut = deps.as_mut();
            cw_ownable::initialize_owner(deps_as_mut.storage, deps_as_mut.api, Some("owner"))
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
            mock_info("stranger", &[]),
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
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
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
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
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
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
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
        let shared_mock_env = mock_env();
        execute(
            deps.as_mut(),
            shared_mock_env.clone(),
            mock_info("owner", &[]),
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
        let shared_mock_env = mock_env();
        execute(
            deps.as_mut(),
            shared_mock_env.clone(),
            mock_info("owner", &[]),
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

mod check_denom {
    use crate::contract::check_denom::DenomType;

    use super::*;

    #[test]
    fn base_denom() {
        let deps = mock_dependencies(&[]);
        let denom_type = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "base_denom",
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap();
        assert_eq!(denom_type, DenomType::Base);
    }

    #[test]
    fn invalid_port() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "icahost/transfer_channel".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn invalid_channel() {
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
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn invalid_port_and_channel() {
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
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn not_an_lsm_share() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "unknown_denom".to_string(),
                        path: "transfer/transfer_channel".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn unknown_validator() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper98765/1".to_string(),
                        path: "transfer/transfer_channel".to_string(),
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
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
        assert!(*query_called.borrow());
    }

    #[test]
    fn invalid_validator_index() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1/2".to_string(),
                        path: "transfer/transfer_channel".to_string(),
                    },
                })
                .unwrap()
            },
        );
        let err = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});
    }

    #[test]
    fn known_validator() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.add_stargate_query_response(
            "/ibc.applications.transfer.v1.Query/DenomTrace",
            |_| {
                to_json_binary(&QueryDenomTraceResponse {
                    denom_trace: DenomTrace {
                        base_denom: "valoper12345/1".to_string(),
                        path: "transfer/transfer_channel".to_string(),
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
        let denom_type = crate::contract::check_denom::check_denom(
            &deps.as_ref(),
            "ibc/12345678",
            &get_default_config(0, 0, 0, 0, 0, 0u128.into()),
        )
        .unwrap();
        assert_eq!(
            denom_type,
            DenomType::LsmShare("valoper12345/1".to_string(), "valoper12345".to_string())
        );
    }
}

mod pending_redeem_shares {
    use crate::contract::get_pending_redeem_msg;

    use super::*;

    #[test]
    fn no_pending_lsm_shares() {
        let mut deps = mock_dependencies(&[]);

        let config = &get_default_config(1000, 3, 100, 100, 600, Uint128::new(100));

        LAST_LSM_REDEEM.save(deps.as_mut().storage, &0).unwrap();

        let redeem_res: Option<CosmosMsg<NeutronMsg>> =
            get_pending_redeem_msg(deps.as_ref(), config, &mock_env(), vec![]).unwrap();

        assert_eq!(redeem_res, None);
    }

    #[test]
    fn lsm_shares_below_threshold() {
        let mut deps = mock_dependencies(&[]);

        let config = &get_default_config(1000, 3, 100, 100, 600, Uint128::new(100));

        let env = &mock_env();

        LSM_SHARES_TO_REDEEM
            .save(
                deps.as_mut().storage,
                "remote_denom_share1".to_string(),
                &(
                    "local_denom_1".to_string(),
                    Uint128::from(100u128),
                    Uint128::from(100u128),
                ),
            )
            .unwrap();

        LAST_LSM_REDEEM
            .save(deps.as_mut().storage, &env.block.time.seconds())
            .unwrap();

        let redeem_res: Option<CosmosMsg<NeutronMsg>> =
            get_pending_redeem_msg(deps.as_ref(), config, env, vec![]).unwrap();

        assert_eq!(redeem_res, None);
    }

    #[test]
    fn lsm_shares_pass_threshold() {
        let mut deps = mock_dependencies(&[]);

        let lsm_redeem_maximum_interval = 100;

        let config = &get_default_config(
            1000,
            3,
            lsm_redeem_maximum_interval,
            100,
            600,
            Uint128::new(100),
        );

        let env = &mock_env();

        LSM_SHARES_TO_REDEEM
            .save(
                deps.as_mut().storage,
                "local_denom_1".to_string(),
                &(
                    "remote_denom_share1".to_string(),
                    Uint128::from(100u128),
                    Uint128::from(100u128),
                ),
            )
            .unwrap();

        LAST_LSM_REDEEM
            .save(
                deps.as_mut().storage,
                &(env.block.time.seconds() - lsm_redeem_maximum_interval * 2),
            )
            .unwrap();

        let redeem_res: Option<CosmosMsg<NeutronMsg>> =
            get_pending_redeem_msg(deps.as_ref(), config, env, vec![]).unwrap();

        assert_eq!(
            redeem_res,
            Some(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_PUPPETEER_CONTRACT_ADDR.to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
                        items: vec![RedeemShareItem {
                            amount: Uint128::from(100u128),
                            local_denom: "local_denom_1".to_string(),
                            remote_denom: "remote_denom_share1".to_string(),
                        }],
                        reply_to: env.contract.address.to_string(),
                    },
                )
                .unwrap(),
                funds: vec![],
            }))
        );
    }

    #[test]
    fn lsm_shares_limit_redeem() {
        let mut deps = mock_dependencies(&[]);

        let lsm_redeem_maximum_interval = 100;

        let config = &get_default_config(
            1000,
            2,
            lsm_redeem_maximum_interval,
            100,
            600,
            Uint128::new(100),
        );

        let env = &mock_env();

        LSM_SHARES_TO_REDEEM
            .save(
                deps.as_mut().storage,
                "local_denom_1".to_string(),
                &(
                    "remote_denom_share1".to_string(),
                    Uint128::from(50u128),
                    Uint128::from(50u128),
                ),
            )
            .unwrap();

        LSM_SHARES_TO_REDEEM
            .save(
                deps.as_mut().storage,
                "local_denom_2".to_string(),
                &(
                    "remote_denom_share2".to_string(),
                    Uint128::from(100u128),
                    Uint128::from(100u128),
                ),
            )
            .unwrap();

        LSM_SHARES_TO_REDEEM
            .save(
                deps.as_mut().storage,
                "local_denom_3".to_string(),
                &(
                    "remote_denom_share3".to_string(),
                    Uint128::from(150u128),
                    Uint128::from(150u128),
                ),
            )
            .unwrap();

        LAST_LSM_REDEEM.save(deps.as_mut().storage, &0).unwrap();

        let redeem_res: Option<CosmosMsg<NeutronMsg>> =
            get_pending_redeem_msg(deps.as_ref(), config, env, vec![]).unwrap();

        assert_eq!(
            redeem_res,
            Some(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_PUPPETEER_CONTRACT_ADDR.to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
                        items: vec![
                            RedeemShareItem {
                                amount: Uint128::from(50u128),
                                local_denom: "local_denom_1".to_string(),
                                remote_denom: "remote_denom_share1".to_string(),
                            },
                            RedeemShareItem {
                                amount: Uint128::from(100u128),
                                local_denom: "local_denom_2".to_string(),
                                remote_denom: "remote_denom_share2".to_string(),
                            }
                        ],
                        reply_to: env.contract.address.to_string(),
                    },
                )
                .unwrap(),
                funds: vec![],
            }))
        );
    }
}
