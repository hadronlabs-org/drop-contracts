use crate::contract::instantiate;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    to_json_binary, Addr, Attribute, Coin, Empty, Event, Response, StdError, Uint128,
};
use cw_multi_test::{custom_app, App, Contract, ContractWrapper, Executor};
use lido_staking_base::msg::rewards_manager::QueryMsg;
use lido_staking_base::msg::rewards_manager::{ExecuteMsg, InstantiateMsg};
use lido_staking_base::state::rewards_manager::HandlerConfig;

const CORE_CONTRACT_ADDR: &str = "core_contract";
const HANDLER_CONTRACT_ADDR: &str = "handler_contract";

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
    let contract: ContractWrapper<Empty, Empty, Empty, StdError, StdError, StdError> =
        ContractWrapper::new(
            |_, _, _, _: Empty| Ok(Response::new()),
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
        Addr::unchecked(CORE_CONTRACT_ADDR),
        &msg,
        &[],
        "rewards manager contract",
        None,
    )
    .unwrap()
}

fn mock_app() -> App {
    custom_app(|_r, _a, _s| {})
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
            handlers: vec![]
        }
    );
}

#[test]
fn test_add_handler() {
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

    let funds: Vec<Coin> = Vec::new();
    let res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            rewards_manager_contract.clone(),
            &ExecuteMsg::AddHandler {
                config: handler_config.clone(),
            },
            &funds,
        )
        .unwrap();

    assert_eq!(
        res.events[1],
        Event::new("crates.io:lido-staking__lido-rewards-manager-add_handler".to_string())
            .add_attributes(vec![
                Attribute::new("denom".to_string(), handler_config.denom.to_string()),
                Attribute::new("address".to_string(), handler_config.address.to_string()),
                Attribute::new(
                    "min_rewards".to_string(),
                    handler_config.min_rewards.to_string()
                ),
            ])
    );

    let config: lido_staking_base::msg::rewards_manager::ConfigResponse = app
        .wrap()
        .query_wasm_smart(rewards_manager_contract.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        lido_staking_base::msg::rewards_manager::ConfigResponse {
            core_address: CORE_CONTRACT_ADDR.to_string(),
            handlers: vec![]
        }
    );
}
