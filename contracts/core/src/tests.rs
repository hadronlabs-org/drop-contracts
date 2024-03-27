use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    to_json_binary, Addr, Coin, CosmosMsg, Decimal, Event, MessageInfo, Order, OwnedDeps, Response,
    StdResult, Timestamp, Uint128, WasmMsg,
};

use drop_helpers::testing::{mock_dependencies, WasmMockQuerier};
use drop_staking_base::{
    msg::strategy::QueryMsg as StategyQueryMsg,
    state::core::{BONDED_AMOUNT, CONFIG},
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
