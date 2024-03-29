use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    to_json_binary, Addr, Coin, CosmosMsg, Decimal, Event, MessageInfo, Order, OwnedDeps, Response,
    StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
};
use std::str::FromStr;

use drop_helpers::testing::{mock_dependencies, WasmMockQuerier};
use drop_puppeteer_base::state::RedeemShareItem;
use drop_staking_base::{
    msg::strategy::QueryMsg as StategyQueryMsg,
    state::core::{
        unbond_batches_map, ContractState, UnbondBatch, UnbondBatchStatus, UnbondItem,
        BONDED_AMOUNT, CONFIG, FSM, LAST_IDLE_CALL, LAST_LSM_REDEEM, LSM_SHARES_TO_REDEEM,
        PENDING_LSM_SHARES, TOTAL_LSM_SHARES, UNBOND_BATCH_ID,
    },
};
use drop_staking_base::{
    msg::{core::InstantiateMsg, puppeteer::MultiBalances},
    state::core::{
        Config, ConfigOptional, FeeItem, NonNativeRewardsItem, COLLECTED_FEES,
        LAST_ICA_BALANCE_CHANGE_HEIGHT, NON_NATIVE_REWARDS_CONFIG,
    },
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v045::types::Balances,
};

use crate::contract::{execute, get_non_native_rewards_and_fee_transfer_msg, get_stake_msg};

pub const MOCK_PUPPETEER_CONTRACT_ADDR: &str = "puppeteer_contract";
pub const MOCK_STRATEGY_CONTRACT_ADDR: &str = "strategy_contract";

fn get_default_config(fee: Option<Decimal>) -> Config {
    Config {
        token_contract: "token_contract".to_string(),
        puppeteer_contract: MOCK_PUPPETEER_CONTRACT_ADDR.to_string(),
        puppeteer_timeout: 60,
        strategy_contract: MOCK_STRATEGY_CONTRACT_ADDR.to_string(),
        withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
        withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
        validators_set_contract: "validators_set_contract".to_string(),
        base_denom: "base_denom".to_string(),
        remote_denom: "remote_denom".to_string(),
        idle_min_interval: 1,
        unbonding_period: 60,
        unbonding_safe_period: 10,
        unbond_batch_switch_time: 6000,
        pump_address: None,
        ld_denom: None,
        channel: "channel".to_string(),
        fee,
        fee_address: Some("fee_address".to_string()),
        lsm_redeem_threshold: 10u64,
        lsm_min_bond_amount: Uint128::one(),
        lsm_redeem_maximum_interval: 10_000_000_000,
        bond_limit: None,
        emergency_address: None,
        min_stake_amount: Uint128::new(100),
    }
}

fn setup_config(deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery>) {
    CONFIG
        .save(
            deps.as_mut().storage,
            &get_default_config(Decimal::from_atomics(1u32, 1).ok()),
        )
        .unwrap();
}

#[test]
fn get_non_native_rewards_and_fee_transfer_msg_success() {
    let mut deps = mock_dependencies(&[]);
    setup_config(&mut deps);
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_msg: &_| {
            to_json_binary(&(
                MultiBalances {
                    coins: vec![Coin {
                        denom: "denom".to_string(),
                        amount: Uint128::new(150),
                    }],
                },
                10u64,
                Timestamp::from_nanos(20),
            ))
            .unwrap()
        });

    NON_NATIVE_REWARDS_CONFIG
        .save(
            deps.as_mut().storage,
            &vec![NonNativeRewardsItem {
                address: "address".to_string(),
                denom: "denom".to_string(),
                min_amount: Uint128::new(100),
                fee: Decimal::from_atomics(1u32, 1).unwrap(),
                fee_address: "fee_address".to_string(),
            }],
        )
        .unwrap();

    let info = mock_info("addr0000", &[Coin::new(1000, "untrn")]);

    let result: CosmosMsg<NeutronMsg> =
        get_non_native_rewards_and_fee_transfer_msg(deps.as_ref(), info, &mock_env())
            .unwrap()
            .unwrap();

    assert_eq!(
        result,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "puppeteer_contract".to_string(),
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
                items: vec![
                    (
                        "address".to_string(),
                        Coin {
                            denom: "denom".to_string(),
                            amount: Uint128::new(135)
                        }
                    ),
                    (
                        "fee_address".to_string(),
                        Coin {
                            denom: "denom".to_string(),
                            amount: Uint128::new(15)
                        }
                    )
                ],
                timeout: Some(60),
                reply_to: "cosmos2contract".to_string()
            })
            .unwrap(),
            funds: vec![Coin::new(1000, "untrn")]
        })
    );
}

#[test]
fn get_non_native_rewards_and_fee_transfer_msg_zero_fee() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_msg: &_| {
            to_json_binary(&(
                MultiBalances {
                    coins: vec![Coin {
                        denom: "denom".to_string(),
                        amount: Uint128::new(150),
                    }],
                },
                10u64,
                Timestamp::from_nanos(20),
            ))
            .unwrap()
        });
    setup_config(&mut deps);

    NON_NATIVE_REWARDS_CONFIG
        .save(
            deps.as_mut().storage,
            &vec![NonNativeRewardsItem {
                address: "address".to_string(),
                denom: "denom".to_string(),
                min_amount: Uint128::new(100),
                fee: Decimal::zero(),
                fee_address: "fee_address".to_string(),
            }],
        )
        .unwrap();

    let info = mock_info("addr0000", &[Coin::new(1000, "untrn")]);

    let result: CosmosMsg<NeutronMsg> =
        get_non_native_rewards_and_fee_transfer_msg(deps.as_ref(), info, &mock_env())
            .unwrap()
            .unwrap();

    assert_eq!(
        result,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "puppeteer_contract".to_string(),
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
                items: vec![(
                    "address".to_string(),
                    Coin {
                        denom: "denom".to_string(),
                        amount: Uint128::new(150)
                    }
                )],
                timeout: Some(60),
                reply_to: "cosmos2contract".to_string()
            })
            .unwrap(),
            funds: vec![Coin::new(1000, "untrn")]
        })
    );
}

#[test]
fn get_stake_msg_success() {
    let mut deps = mock_dependencies(&[]);
    setup_config(&mut deps);
    LAST_ICA_BALANCE_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &1)
        .unwrap();
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_msg: &_| {
            to_json_binary(&(
                Balances {
                    coins: vec![Coin {
                        denom: "remote_denom".to_string(),
                        amount: Uint128::new(200),
                    }],
                },
                10u64,
                Timestamp::from_nanos(20),
            ))
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StategyQueryMsg = from_json(msg).unwrap();
            match q {
                StategyQueryMsg::CalcDeposit { deposit } => to_json_binary(&vec![
                    drop_staking_base::msg::distribution::IdealDelegation {
                        valoper_address: "valoper_address".to_string(),
                        stake_change: deposit,
                        ideal_stake: deposit,
                        current_stake: deposit,
                        weight: 1u64,
                    },
                ])
                .unwrap(),
                _ => unimplemented!(),
            }
        });

    let stake_msg: CosmosMsg<NeutronMsg> = get_stake_msg(
        deps.as_mut(),
        &mock_env(),
        &get_default_config(Decimal::from_atomics(1u32, 1).ok()),
        &MessageInfo {
            sender: Addr::unchecked("addr0000"),
            funds: vec![Coin::new(200, "untrn")],
        },
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        stake_msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "puppeteer_contract".to_string(),
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
                items: vec![("valoper_address".to_string(), Uint128::new(180))],
                timeout: Some(60),
                reply_to: "cosmos2contract".to_string(),
            })
            .unwrap(),
            funds: vec![Coin::new(200, "untrn")],
        })
    );

    let collected_fees = COLLECTED_FEES
        .range_raw(deps.as_mut().storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect::<StdResult<Vec<FeeItem>>>()
        .unwrap();

    assert_eq!(
        collected_fees[0],
        FeeItem {
            address: "fee_address".to_string(),
            denom: "remote_denom".to_string(),
            amount: Uint128::new(20),
        }
    );
}

#[test]
fn get_stake_msg_zero_fee() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_msg: &_| {
            to_json_binary(&(
                Balances {
                    coins: vec![Coin {
                        denom: "remote_denom".to_string(),
                        amount: Uint128::new(200),
                    }],
                },
                10u64,
                Timestamp::from_nanos(20),
            ))
            .unwrap()
        });
    deps.querier
        .add_wasm_query_response("strategy_contract", |msg| {
            let q: StategyQueryMsg = from_json(msg).unwrap();
            match q {
                StategyQueryMsg::CalcDeposit { deposit } => to_json_binary(&vec![
                    drop_staking_base::msg::distribution::IdealDelegation {
                        valoper_address: "valoper_address".to_string(),
                        stake_change: deposit,
                        ideal_stake: deposit,
                        current_stake: deposit,
                        weight: 1u64,
                    },
                ])
                .unwrap(),
                _ => unimplemented!(),
            }
        });
    setup_config(&mut deps);
    LAST_ICA_BALANCE_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &1)
        .unwrap();

    let stake_msg: CosmosMsg<NeutronMsg> = get_stake_msg(
        deps.as_mut(),
        &mock_env(),
        &get_default_config(None),
        &MessageInfo {
            sender: Addr::unchecked("addr0000"),
            funds: vec![Coin::new(200, "untrn")],
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        stake_msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "puppeteer_contract".to_string(),
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
                items: vec![("valoper_address".to_string(), Uint128::new(200))],
                timeout: Some(60),
                reply_to: "cosmos2contract".to_string(),
            })
            .unwrap(),
            funds: vec![Coin::new(200, "untrn")],
        })
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
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
            puppeteer_timeout: 10,
            strategy_contract: "old_strategy_contract".to_string(),
            withdrawal_voucher_contract: "old_withdrawal_voucher_contract".to_string(),
            withdrawal_manager_contract: "old_withdrawal_manager_contract".to_string(),
            validators_set_contract: "old_validators_set_contract".to_string(),
            base_denom: "old_base_denom".to_string(),
            remote_denom: "old_remote_denom".to_string(),
            idle_min_interval: 12,
            unbonding_period: 20,
            unbonding_safe_period: 120,
            unbond_batch_switch_time: 2000,
            pump_address: Some("old_pump_address".to_string()),
            channel: "old_channel".to_string(),
            fee: Some(Decimal::from_atomics(2u32, 1).unwrap()),
            fee_address: Some("old_fee_address".to_string()),
            lsm_redeem_max_interval: 20_000_000,
            lsm_redeem_threshold: 120u64,
            lsm_min_bond_amount: Uint128::new(12),
            bond_limit: Some(Uint128::new(12)),
            emergency_address: Some("old_emergency_address".to_string()),
            min_stake_amount: Uint128::new(1200),
            owner: "admin".to_string(),
        },
    )
    .unwrap();

    let new_config = ConfigOptional {
        token_contract: Some("new_token_contract".to_string()),
        puppeteer_contract: Some("new_puppeteer_contract".to_string()),
        puppeteer_timeout: Some(100),
        strategy_contract: Some("new_strategy_contract".to_string()),
        withdrawal_voucher_contract: Some("new_withdrawal_voucher_contract".to_string()),
        withdrawal_manager_contract: Some("new_withdrawal_manager_contract".to_string()),
        validators_set_contract: Some("new_validators_set_contract".to_string()),
        base_denom: Some("new_base_denom".to_string()),
        remote_denom: Some("new_remote_denom".to_string()),
        idle_min_interval: Some(2),
        unbonding_period: Some(120),
        unbonding_safe_period: Some(20),
        unbond_batch_switch_time: Some(12000),
        pump_address: Some("new_pump_address".to_string()),
        ld_denom: Some("new_ld_denom".to_string()),
        channel: Some("new_channel".to_string()),
        fee: Some(Decimal::from_atomics(2u32, 1).unwrap()),
        fee_address: Some("new_fee_address".to_string()),
        lsm_redeem_threshold: Some(20u64),
        lsm_min_bond_amount: Some(Uint128::new(2)),
        lsm_redeem_maximum_interval: Some(20_000_000_000),
        bond_limit: Some(Uint128::new(2)),
        emergency_address: Some("new_emergency_address".to_string()),
        min_stake_amount: Some(Uint128::new(200)),
    };
    let expected_config = Config {
        token_contract: "new_token_contract".to_string(),
        puppeteer_contract: "new_puppeteer_contract".to_string(),
        puppeteer_timeout: 100,
        strategy_contract: "new_strategy_contract".to_string(),
        withdrawal_voucher_contract: "new_withdrawal_voucher_contract".to_string(),
        withdrawal_manager_contract: "new_withdrawal_manager_contract".to_string(),
        validators_set_contract: "new_validators_set_contract".to_string(),
        base_denom: "new_base_denom".to_string(),
        remote_denom: "new_remote_denom".to_string(),
        idle_min_interval: 2,
        unbonding_period: 120,
        unbonding_safe_period: 20,
        unbond_batch_switch_time: 12000,
        pump_address: Some("new_pump_address".to_string()),
        ld_denom: Some("new_ld_denom".to_string()),
        channel: "new_channel".to_string(),
        fee: Some(Decimal::from_atomics(2u32, 1).unwrap()),
        fee_address: Some("new_fee_address".to_string()),
        lsm_redeem_threshold: 20u64,
        lsm_min_bond_amount: Uint128::new(2),
        lsm_redeem_maximum_interval: 20_000_000_000,
        bond_limit: Some(Uint128::new(2)),
        emergency_address: Some("new_emergency_address".to_string()),
        min_stake_amount: Uint128::new(200),
    };

    let res = crate::contract::execute(
        deps_mut,
        env.clone(),
        info,
        drop_staking_base::msg::core::ExecuteMsg::UpdateConfig {
            new_config: Box::new(new_config),
        },
    );
    assert!(res.is_ok());
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, expected_config);
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
        drop_staking_base::msg::core::ExecuteMsg::ResetBondedAmount {},
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
fn test_execute_tick_idle_non_native_rewards() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&(
                MultiBalances {
                    coins: vec![
                        Coin {
                            denom: "non_native_denom_1".to_string(),
                            amount: Uint128::new(200),
                        },
                        Coin {
                            denom: "non_native_denom_2".to_string(),
                            amount: Uint128::new(200),
                        },
                        Coin {
                            denom: "non_native_denom_3".to_string(),
                            amount: Uint128::new(200),
                        },
                        Coin {
                            denom: "non_native_denom_4".to_string(),
                            amount: Uint128::new(99),
                        },
                    ],
                },
                10u64,
                Timestamp::from_nanos(20),
            ))
            .unwrap()
        });

    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                token_contract: "token_contract".to_string(),
                puppeteer_contract: "puppeteer_contract".to_string(),
                puppeteer_timeout: 60,
                strategy_contract: "strategy_contract".to_string(),
                withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
                withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
                validators_set_contract: "validators_set_contract".to_string(),
                base_denom: "base_denom".to_string(),
                remote_denom: "remote_denom".to_string(),
                idle_min_interval: 1000,
                unbonding_period: 60,
                unbonding_safe_period: 10,
                unbond_batch_switch_time: 6000,
                pump_address: Some("pump_address".to_string()),
                ld_denom: Some("ld_denom".to_string()),
                channel: "channel".to_string(),
                fee: Some(Decimal::from_atomics(1u32, 1).unwrap()),
                fee_address: Some("fee_address".to_string()),
                lsm_redeem_threshold: 10u64,
                lsm_min_bond_amount: Uint128::one(),
                lsm_redeem_maximum_interval: 10_000_000_000,
                bond_limit: None,
                emergency_address: None,
                min_stake_amount: Uint128::new(100),
            },
        )
        .unwrap();
    NON_NATIVE_REWARDS_CONFIG
        .save(
            deps.as_mut().storage,
            &vec![
                NonNativeRewardsItem {
                    denom: "non_native_denom_1".to_string(),
                    address: "non_native_denom_receiver_1".to_string(),
                    min_amount: Uint128::new(100),
                    fee_address: "non_native_denom_fee_receiver_1".to_string(),
                    fee: Decimal::from_str("0.1").unwrap(),
                },
                NonNativeRewardsItem {
                    denom: "non_native_denom_2".to_string(),
                    address: "non_native_denom_receiver_2".to_string(),
                    min_amount: Uint128::new(100),
                    fee_address: "non_native_denom_fee_receiver_2".to_string(),
                    fee: Decimal::from_str("1").unwrap(),
                },
                NonNativeRewardsItem {
                    denom: "non_native_denom_3".to_string(),
                    address: "non_native_denom_receiver_3".to_string(),
                    min_amount: Uint128::new(100),
                    fee_address: "non_native_denom_fee_receiver_3".to_string(),
                    fee: Decimal::from_str("0").unwrap(),
                },
                NonNativeRewardsItem {
                    denom: "non_native_denom_4".to_string(),
                    address: "non_native_denom_receiver_4".to_string(),
                    min_amount: Uint128::new(100),
                    fee_address: "non_native_denom_fee_receiver_4".to_string(),
                    fee: Decimal::from_str("0").unwrap(),
                },
            ],
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_IDLE_CALL.save(deps.as_mut().storage, &0).unwrap();
    LAST_ICA_BALANCE_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    LAST_LSM_REDEEM.save(deps.as_mut().storage, &0).unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[]),
        drop_staking_base::msg::core::ExecuteMsg::Tick {},
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-tick_idle")
                    .add_attributes(vec![("action", "tick_idle"),])
            )
            .add_submessages(vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
                    items: vec![
                        (
                            "non_native_denom_receiver_1".to_string(),
                            cosmwasm_std::Coin {
                                denom: "non_native_denom_1".to_string(),
                                amount: Uint128::from(180u128),
                            },
                        ),
                        (
                            "non_native_denom_fee_receiver_1".to_string(),
                            cosmwasm_std::Coin {
                                denom: "non_native_denom_1".to_string(),
                                amount: Uint128::from(20u128),
                            },
                        ),
                        (
                            "non_native_denom_fee_receiver_2".to_string(),
                            cosmwasm_std::Coin {
                                denom: "non_native_denom_2".to_string(),
                                amount: Uint128::from(200u128),
                            },
                        ),
                        (
                            "non_native_denom_receiver_3".to_string(),
                            cosmwasm_std::Coin {
                                denom: "non_native_denom_3".to_string(),
                                amount: Uint128::from(200u128),
                            },
                        ),
                    ],
                    timeout: Some(60),
                    reply_to: "cosmos2contract".to_string(),
                })
                .unwrap(),
                funds: vec![]
            }))])
    );
}

#[test]
fn test_execute_tick_idle_get_pending_lsm_shares_transfer() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                token_contract: "token_contract".to_string(),
                puppeteer_contract: "puppeteer_contract".to_string(),
                puppeteer_timeout: 60,
                strategy_contract: "strategy_contract".to_string(),
                withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
                withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
                validators_set_contract: "validators_set_contract".to_string(),
                base_denom: "base_denom".to_string(),
                remote_denom: "remote_denom".to_string(),
                idle_min_interval: 1000,
                unbonding_period: 60,
                unbonding_safe_period: 10,
                unbond_batch_switch_time: 6000,
                pump_address: Some("pump_address".to_string()),
                ld_denom: Some("ld_denom".to_string()),
                channel: "channel".to_string(),
                fee: Some(Decimal::from_atomics(1u32, 1).unwrap()),
                fee_address: Some("fee_address".to_string()),
                lsm_redeem_threshold: 10u64,
                lsm_min_bond_amount: Uint128::one(),
                lsm_redeem_maximum_interval: 10_000_000_000,
                bond_limit: None,
                emergency_address: None,
                min_stake_amount: Uint128::new(100),
            },
        )
        .unwrap();
    NON_NATIVE_REWARDS_CONFIG
        .save(deps.as_mut().storage, &vec![])
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_IDLE_CALL.save(deps.as_mut().storage, &0).unwrap();
    LAST_ICA_BALANCE_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    LAST_LSM_REDEEM.save(deps.as_mut().storage, &0).unwrap();
    PENDING_LSM_SHARES
        .save(
            deps.as_mut().storage,
            "remote_denom".to_string(),
            &("local_denom".to_string(), Uint128::from(100u128)),
        )
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[]),
        drop_staking_base::msg::core::ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-tick_idle")
                    .add_attribute("action", "tick_idle")
            )
            .add_submessages(vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "puppeteer_contract".to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
                        timeout: 60,
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
            &Config {
                token_contract: "token_contract".to_string(),
                puppeteer_contract: "puppeteer_contract".to_string(),
                puppeteer_timeout: 60,
                strategy_contract: "strategy_contract".to_string(),
                withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
                withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
                validators_set_contract: "validators_set_contract".to_string(),
                base_denom: "base_denom".to_string(),
                remote_denom: "remote_denom".to_string(),
                idle_min_interval: 1000,
                unbonding_period: 60,
                unbonding_safe_period: 10,
                unbond_batch_switch_time: 6000,
                pump_address: Some("pump_address".to_string()),
                ld_denom: Some("ld_denom".to_string()),
                channel: "channel".to_string(),
                fee: Some(Decimal::from_atomics(1u32, 1).unwrap()),
                fee_address: Some("fee_address".to_string()),
                lsm_redeem_threshold: 3u64,
                lsm_min_bond_amount: Uint128::one(),
                lsm_redeem_maximum_interval: 100,
                bond_limit: None,
                emergency_address: None,
                min_stake_amount: Uint128::new(100),
            },
        )
        .unwrap();
    NON_NATIVE_REWARDS_CONFIG
        .save(deps.as_mut().storage, &vec![])
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_IDLE_CALL.save(deps.as_mut().storage, &0).unwrap();
    LAST_ICA_BALANCE_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &0)
        .unwrap();
    TOTAL_LSM_SHARES.save(deps.as_mut().storage, &0).unwrap();
    BONDED_AMOUNT
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    LAST_LSM_REDEEM.save(deps.as_mut().storage, &0).unwrap();
    LSM_SHARES_TO_REDEEM
        .save(
            deps.as_mut().storage,
            "remote_denom_share1".to_string(),
            &("local_denom_1".to_string(), Uint128::from(100u128)),
        )
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("admin", &[]),
        drop_staking_base::msg::core::ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    LSM_SHARES_TO_REDEEM
        .save(
            deps.as_mut().storage,
            "remote_denom_share2".to_string(),
            &("local_denom_2".to_string(), Uint128::from(100u128)),
        )
        .unwrap();
    LSM_SHARES_TO_REDEEM
        .save(
            deps.as_mut().storage,
            "remote_denom_share3".to_string(),
            &("local_denom_3".to_string(), Uint128::from(100u128)),
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[]),
        drop_staking_base::msg::core::ExecuteMsg::Tick {},
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-core-execute-tick_idle")
                    .add_attribute("action", "tick_idle")
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
                        timeout: Some(60),
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
                Timestamp::from_nanos(20),
            ))
            .unwrap()
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
    deps.querier
        .add_wasm_query_response("puppeteer_contract", |_| {
            to_json_binary(&(
                neutron_sdk::interchain_queries::v045::types::Delegations {
                    delegations: vec![cosmwasm_std::Delegation {
                        delegator: Addr::unchecked("ica_address"),
                        validator: "valoper_address".to_string(),
                        amount: cosmwasm_std::Coin {
                            denom: "remote_denom".to_string(),
                            amount: Uint128::new(100_000),
                        },
                    }],
                },
                0u64,
                Timestamp::from_seconds(0),
            ))
            .unwrap()
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                token_contract: "token_contract".to_string(),
                puppeteer_contract: "puppeteer_contract".to_string(),
                puppeteer_timeout: 60,
                strategy_contract: "strategy_contract".to_string(),
                withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
                withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
                validators_set_contract: "validators_set_contract".to_string(),
                base_denom: "base_denom".to_string(),
                remote_denom: "remote_denom".to_string(),
                idle_min_interval: 1000,
                unbonding_period: 60,
                unbonding_safe_period: 100,
                unbond_batch_switch_time: 6000,
                pump_address: Some("pump_address".to_string()),
                ld_denom: Some("ld_denom".to_string()),
                channel: "channel".to_string(),
                fee: Some(Decimal::from_atomics(1u32, 1).unwrap()),
                fee_address: Some("fee_address".to_string()),
                lsm_redeem_threshold: 3u64,
                lsm_min_bond_amount: Uint128::one(),
                lsm_redeem_maximum_interval: 100,
                bond_limit: None,
                emergency_address: None,
                min_stake_amount: Uint128::new(100),
            },
        )
        .unwrap();
    FSM.set_initial_state(deps.as_mut().storage, ContractState::Idle)
        .unwrap();
    LAST_IDLE_CALL.save(deps.as_mut().storage, &0).unwrap();
    LAST_ICA_BALANCE_CHANGE_HEIGHT
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
                total_amount: Uint128::from(1000u128),
                expected_amount: Uint128::from(1000u128),
                unbond_items: vec![UnbondItem {
                    amount: Uint128::from(1000u128),
                    sender: "some_sender".to_string(),
                    expected_amount: Uint128::from(1000u128),
                }],
                status: UnbondBatchStatus::Unbonding,
                expected_release: 10001,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawed_amount: None,
                created: 1,
            },
        )
        .unwrap();
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(10000);
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("admin", &[]),
        drop_staking_base::msg::core::ExecuteMsg::Tick {},
    );
    assert!(res.is_err());
    assert_eq!(
        res,
        Err(crate::error::ContractError::UnbondingTimeIsClose {})
    );
}
