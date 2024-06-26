use std::marker::PhantomData;

use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, Coin, ContractResult, CosmosMsg, Decimal, Empty, MessageInfo, Order,
    OwnedDeps, Querier, QuerierResult, QueryRequest, StdResult, SystemError, SystemResult,
    Timestamp, Uint128, WasmMsg, WasmQuery,
};

use drop_puppeteer_base::msg::QueryMsg as PuppeteerBaseQueryMsg;
use drop_staking_base::{
    msg::puppeteer::{MultiBalances, QueryExtMsg},
    state::core::{
        Config, FeeItem, NonNativeRewardsItem, COLLECTED_FEES, LAST_ICA_BALANCE_CHANGE_HEIGHT,
        NON_NATIVE_REWARDS_CONFIG,
    },
};
use drop_staking_base::{msg::strategy::QueryMsg as StategyQueryMsg, state::core::CONFIG};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v045::types::Balances,
};

use crate::contract::{get_non_native_rewards_and_fee_transfer_msg, get_stake_msg};

pub const MOCK_PUPPETEER_CONTRACT_ADDR: &str = "puppeteer_contract";
pub const MOCK_STRATEGY_CONTRACT_ADDR: &str = "strategy_contract";

fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery> {
    let custom_querier = WasmMockQuerier::new(MockQuerier::new(&[]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if contract_addr == MOCK_PUPPETEER_CONTRACT_ADDR {
                    let q: PuppeteerBaseQueryMsg<QueryExtMsg> = from_json(msg).unwrap();
                    let reply = match q {
                        PuppeteerBaseQueryMsg::Extention { msg } => match msg {
                            QueryExtMsg::NonNativeRewardsBalances {} => {
                                let data = (
                                    MultiBalances {
                                        coins: vec![Coin {
                                            denom: "denom".to_string(),
                                            amount: Uint128::new(150),
                                        }],
                                    },
                                    10u64,
                                    Timestamp::from_nanos(20),
                                );
                                to_json_binary(&data)
                            }
                            QueryExtMsg::Balances {} => {
                                let data = (
                                    Balances {
                                        coins: vec![Coin {
                                            denom: "remote_denom".to_string(),
                                            amount: Uint128::new(200),
                                        }],
                                    },
                                    10u64,
                                    Timestamp::from_nanos(20),
                                );
                                to_json_binary(&data)
                            }
                            _ => todo!(),
                        },
                        _ => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(reply));
                }
                if contract_addr == MOCK_STRATEGY_CONTRACT_ADDR {
                    let q: StategyQueryMsg = from_json(msg).unwrap();
                    let reply = match q {
                        StategyQueryMsg::CalcDeposit { deposit } => to_json_binary(&vec![
                            drop_staking_base::msg::distribution::IdealDelegation {
                                valoper_address: "valoper_address".to_string(),
                                stake_change: deposit,
                                ideal_stake: deposit,
                                current_stake: deposit,
                                weight: 1u64,
                            },
                        ]),
                        _ => todo!(),
                    };
                    return SystemResult::Ok(ContractResult::from(reply));
                }
                SystemResult::Err(SystemError::NoSuchContract {
                    addr: contract_addr.to_string(),
                })
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier) -> WasmMockQuerier {
        WasmMockQuerier { base }
    }
}

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
    let mut deps = mock_dependencies();

    setup_config(&mut deps);

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
    let mut deps = mock_dependencies();

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
    let mut deps = mock_dependencies();

    setup_config(&mut deps);

    LAST_ICA_BALANCE_CHANGE_HEIGHT
        .save(deps.as_mut().storage, &1)
        .unwrap();

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
    let mut deps = mock_dependencies();

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
