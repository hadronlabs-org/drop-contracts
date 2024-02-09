use crate::contract::instantiate;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    attr, coins, to_json_binary, Addr, Attribute, Coin, Empty, Event, Response, StdError, Uint128,
};
use cw_multi_test::{custom_app, App, Contract, ContractWrapper, Executor};
use lido_helpers::answer::{attr_coin, response};
use lido_staking_base::msg::reward_handler::HandlerExecuteMsg;
use lido_staking_base::msg::rewards_manager::QueryMsg;
use lido_staking_base::msg::rewards_manager::{ExecuteMsg, InstantiateMsg};
use lido_staking_base::state::rewards_manager::HandlerConfig;

const CORE_CONTRACT_ADDR: &str = "core_contract";

const SENDER_ADDR: &str = "sender";

fn instantiate_contract(
    app: &mut App,
    contract: fn() -> Box<dyn Contract<Empty>>,
    label: String,
) -> Addr {
    let contract_id = app.store_code(contract());
    app.instantiate_contract(
        contract_id,
        Addr::unchecked(CORE_CONTRACT_ADDR),
        &Empty {},
        &[],
        label,
        None,
    )
    .unwrap()
}

fn handler_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<HandlerExecuteMsg, Empty, Empty, StdError, StdError, StdError> =
        ContractWrapper::new(
            |_, _, info, msg: HandlerExecuteMsg| {
                match msg {
                    HandlerExecuteMsg::Exchange {} => {
                        if !info.funds.is_empty() {
                            println!("received funds {:?}", info.funds);
                            return Ok(response(
                                "handler_contract_execute",
                                "handler_mock",
                                [
                                    attr("message", "ExecuteMsg::Exchange".to_string()),
                                    attr_coin(
                                        "received_funds",
                                        info.funds[0].amount.to_string(),
                                        info.funds[0].denom.clone(),
                                    ),
                                ],
                            ));
                        }
                    }
                }

                println!("handler contract execute");

                Err(StdError::generic_err("Wrong execution call"))
            },
            |_, _, _, _: Empty| Ok(Response::new()),
            |_, _, _: Empty| to_json_binary(&{}),
        );
    Box::new(contract)
}

fn instantiate_handler_contract(app: &mut App) -> Addr {
    instantiate_contract(app, handler_contract, "lido handler contract".to_string())
}

fn rewards_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn instantiate_rewards_manager_contract(app: &mut App, id: u64, msg: InstantiateMsg) -> Addr {
    app.instantiate_contract(
        id,
        Addr::unchecked("sender"),
        &msg,
        &[],
        "rewards manager contract",
        None,
    )
    .unwrap()
}

fn mock_app() -> App {
    custom_app(|r, _a, s| {
        r.bank
            .init_balance(
                s,
                &Addr::unchecked(SENDER_ADDR),
                vec![
                    Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(1000000000u128),
                    },
                    Coin {
                        denom: "ueth".to_string(),
                        amount: Uint128::from(1000000000u128),
                    },
                ],
            )
            .unwrap();
    })
}

#[test]
fn test_initialization() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        core_address: CORE_CONTRACT_ADDR.to_string(),
    };

    let info = mock_info(CORE_CONTRACT_ADDR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("crates.io:lido-staking__lido-rewards-manager-instantiate".to_string())
                .add_attributes(vec![Attribute::new(
                    "core_address".to_string(),
                    CORE_CONTRACT_ADDR.to_string()
                ),])
        ]
    );
}

#[test]
fn test_config_query() {
    let mut app = mock_app();

    let rewards_manager_code_id = app.store_code(rewards_manager_contract());

    let rewards_manager_contract = instantiate_rewards_manager_contract(
        &mut app,
        rewards_manager_code_id,
        InstantiateMsg {
            core_address: CORE_CONTRACT_ADDR.to_string(),
        },
    );

    let config: lido_staking_base::msg::rewards_manager::ConfigResponse = app
        .wrap()
        .query_wasm_smart(rewards_manager_contract.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        lido_staking_base::msg::rewards_manager::ConfigResponse {
            core_address: CORE_CONTRACT_ADDR.to_string(),
        }
    );
}

#[test]
fn test_handlers_query() {
    let mut app = mock_app();

    let rewards_manager_code_id = app.store_code(rewards_manager_contract());

    let rewards_manager_contract = instantiate_rewards_manager_contract(
        &mut app,
        rewards_manager_code_id,
        InstantiateMsg {
            core_address: CORE_CONTRACT_ADDR.to_string(),
        },
    );

    let handlers: Vec<HandlerConfig> = app
        .wrap()
        .query_wasm_smart(rewards_manager_contract.clone(), &QueryMsg::Handlers {})
        .unwrap();

    assert_eq!(handlers, vec![]);
}

#[test]
fn test_add_remove_handler() {
    let mut app = mock_app();

    let handler_contract = instantiate_handler_contract(&mut app);

    let rewards_manager_code_id = app.store_code(rewards_manager_contract());

    let rewards_manager_contract = instantiate_rewards_manager_contract(
        &mut app,
        rewards_manager_code_id,
        InstantiateMsg {
            core_address: CORE_CONTRACT_ADDR.to_string(),
        },
    );

    let handler_config = HandlerConfig {
        address: handler_contract.to_string(),
        denom: "ueth".to_string(),
        min_rewards: Uint128::zero(),
    };

    let res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::AddHandler {
                config: handler_config.clone(),
            },
            &[],
        )
        .unwrap();

    let ty = res.events[1].ty.clone();

    assert_eq!(
        ty,
        "wasm-crates.io:lido-staking__lido-rewards-manager-add_handler".to_string()
    );

    let attrs = res.events[1].attributes[1..].to_vec();

    assert_eq!(
        attrs,
        vec![
            Attribute::new("denom".to_string(), handler_config.denom.to_string()),
            Attribute::new("address".to_string(), handler_config.address.to_string()),
            Attribute::new(
                "min_rewards".to_string(),
                handler_config.min_rewards.to_string()
            ),
        ]
    );

    let handlers: Vec<HandlerConfig> = app
        .wrap()
        .query_wasm_smart(rewards_manager_contract.clone(), &QueryMsg::Handlers {})
        .unwrap();

    assert_eq!(
        handlers,
        vec![HandlerConfig {
            address: handler_config.address.clone(),
            denom: handler_config.denom.clone(),
            min_rewards: Uint128::zero()
        }]
    );

    let res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::RemoveHandler {
                denom: handler_config.denom.clone(),
            },
            &[],
        )
        .unwrap();

    let ty = res.events[1].ty.clone();

    assert_eq!(
        ty,
        "wasm-crates.io:lido-staking__lido-rewards-manager-remove_handler".to_string()
    );

    let attrs = res.events[1].attributes[1..].to_vec();

    assert_eq!(
        attrs,
        vec![Attribute::new(
            "denom".to_string(),
            handler_config.denom.to_string()
        ),]
    );

    let handlers: Vec<HandlerConfig> = app
        .wrap()
        .query_wasm_smart(rewards_manager_contract.clone(), &QueryMsg::Handlers {})
        .unwrap();

    assert_eq!(handlers, vec![]);
}

#[test]
fn test_handler_call() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let handler_contract = instantiate_handler_contract(&mut app);

    let rewards_manager_code_id = app.store_code(rewards_manager_contract());

    let rewards_manager_contract = instantiate_rewards_manager_contract(
        &mut app,
        rewards_manager_code_id,
        InstantiateMsg {
            core_address: CORE_CONTRACT_ADDR.to_string(),
        },
    );

    let amount = coins(100, "ueth");
    let _ = app
        .send_tokens(sender_address, rewards_manager_contract.clone(), &amount)
        .unwrap();

    let handler_config = HandlerConfig {
        address: handler_contract.to_string(),
        denom: "ueth".to_string(),
        min_rewards: Uint128::zero(),
    };

    let _res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::AddHandler {
                config: handler_config.clone(),
            },
            &[],
        )
        .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::ExchangeRewards {},
            &[],
        )
        .unwrap();

    let ty = res.events[4].ty.clone();

    assert_eq!(ty, "wasm-handler_mock-handler_contract_execute".to_string());

    let attrs = res.events[4].attributes[1..].to_vec();

    assert_eq!(
        attrs,
        vec![
            Attribute::new("message".to_string(), "ExecuteMsg::Exchange".to_string()),
            Attribute::new("received_funds".to_string(), "100ueth".to_string()),
        ]
    );
}

#[test]
fn test_two_handlers_call() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let ueth_handler_contract = instantiate_handler_contract(&mut app);
    let untrn_handler_contract = instantiate_handler_contract(&mut app);

    let rewards_manager_code_id = app.store_code(rewards_manager_contract());

    let rewards_manager_contract = instantiate_rewards_manager_contract(
        &mut app,
        rewards_manager_code_id,
        InstantiateMsg {
            core_address: CORE_CONTRACT_ADDR.to_string(),
        },
    );

    let amount = coins(100, "ueth");
    let _ = app
        .send_tokens(
            sender_address.clone(),
            rewards_manager_contract.clone(),
            &amount,
        )
        .unwrap();

    let amount = coins(55, "untrn");
    let _ = app
        .send_tokens(sender_address, rewards_manager_contract.clone(), &amount)
        .unwrap();

    let ueth_handler_config = HandlerConfig {
        address: ueth_handler_contract.to_string(),
        denom: "ueth".to_string(),
        min_rewards: Uint128::zero(),
    };

    let untrn_handler_config = HandlerConfig {
        address: untrn_handler_contract.to_string(),
        denom: "untrn".to_string(),
        min_rewards: Uint128::zero(),
    };

    let _res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::AddHandler {
                config: ueth_handler_config.clone(),
            },
            &[],
        )
        .unwrap();

    let _res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::AddHandler {
                config: untrn_handler_config.clone(),
            },
            &[],
        )
        .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::ExchangeRewards {},
            &[],
        )
        .unwrap();

    let ty = res.events[4].ty.clone();

    assert_eq!(ty, "wasm-handler_mock-handler_contract_execute".to_string());

    let ueth_attrs = res.events[4].attributes[1..].to_vec();

    assert_eq!(
        ueth_attrs,
        vec![
            Attribute::new("message".to_string(), "ExecuteMsg::Exchange".to_string()),
            Attribute::new("received_funds".to_string(), "100ueth".to_string()),
        ]
    );

    let untrn_attrs = res.events[6].attributes[1..].to_vec();

    assert_eq!(
        untrn_attrs,
        vec![
            Attribute::new("message".to_string(), "ExecuteMsg::Exchange".to_string()),
            Attribute::new("received_funds".to_string(), "55untrn".to_string()),
        ]
    );
}
