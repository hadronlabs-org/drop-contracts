use crate::contract::instantiate;

use astroport::asset::AssetInfo;
use astroport::pair::ExecuteMsg as PairExecuteMsg;
use astroport::router::{ExecuteMsg as RouterExecuteMsg, SwapOperation};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    attr, coins, to_json_binary, Addr, Attribute, Coin, Empty, Event, Response, StdError,
    StdResult, Uint128,
};
use cw_multi_test::{custom_app, App, Contract, ContractWrapper, Executor};
use drop_helpers::answer::response;
use drop_staking_base::msg::astroport_exchange_handler::{
    ConfigResponse, ExecuteMsg, InstantiateMsg,
};
use drop_staking_base::msg::rewards_manager::QueryMsg;

const CORE_CONTRACT_ADDR: &str = "core_contract";
const OWNER_CONTRACT_ADDR: &str = "owner_contract";
const CRON_ADDR: &str = "cron_address";

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

fn pair_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<PairExecuteMsg, Empty, Empty, StdError, StdError, StdError> =
        ContractWrapper::new(
            |_, _, info, msg: PairExecuteMsg| {
                match msg {
                    PairExecuteMsg::Swap {
                        offer_asset,
                        ask_asset_info,
                        belief_price,
                        max_spread,
                        to,
                    } => {
                        if !info.funds.is_empty() {
                            let asset_denom = match offer_asset.info {
                                AssetInfo::NativeToken { denom } => denom,
                                _ => {
                                    return Err(StdError::generic_err("Wrong token type"));
                                }
                            };

                            return Ok(response(
                                "pair_contract_execute",
                                "pair_mock",
                                [
                                    attr("message", "PairExecuteMsg::Swap".to_string()),
                                    attr("to", to.unwrap().to_string()),
                                    attr("ask_asset_info", ask_asset_info.is_some().to_string()),
                                    attr("belief_price", belief_price.is_some().to_string()),
                                    attr("max_spread", max_spread.is_some().to_string()),
                                    attr(
                                        "offer_asset",
                                        format!("{}{}", offer_asset.amount, asset_denom),
                                    ),
                                    attr(
                                        "funds_received",
                                        format!("{}{}", info.funds[0].amount, info.funds[0].denom),
                                    ),
                                ],
                            ));
                        }
                    }
                    _ => {
                        return Err(StdError::generic_err("Wrong execution call"));
                    }
                }

                Err(StdError::generic_err("Wrong execution call"))
            },
            |_, _, _, _: Empty| Ok(Response::new()),
            |_, _, _: Empty| to_json_binary(&{}),
        );
    Box::new(contract)
}

fn instantiate_pair_contract(app: &mut App) -> Addr {
    instantiate_contract(app, pair_contract, "astroport pair contract".to_string())
}

fn router_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<RouterExecuteMsg, Empty, Empty, StdError, StdError, StdError> =
        ContractWrapper::new(
            |_, _, info, msg: RouterExecuteMsg| {
                match msg {
                    RouterExecuteMsg::ExecuteSwapOperations {
                        operations,
                        minimum_receive,
                        max_spread,
                        to,
                    } => {
                        if !info.funds.is_empty() {
                            return Ok(response(
                                "router_contract_execute",
                                "router_mock",
                                [
                                    attr(
                                        "message",
                                        "RouterExecuteMsg::ExecuteSwapOperations".to_string(),
                                    ),
                                    attr("to", to.unwrap().to_string()),
                                    attr("minimum_receive", minimum_receive.is_some().to_string()),
                                    attr("max_spread", max_spread.is_some().to_string()),
                                    attr(
                                        "funds_received",
                                        format!("{}{}", info.funds[0].amount, info.funds[0].denom),
                                    ),
                                    attr(
                                        "operation1",
                                        get_swap_operation(operations[0].clone()).unwrap(),
                                    ),
                                    attr(
                                        "operation2",
                                        get_swap_operation(operations[1].clone()).unwrap(),
                                    ),
                                ],
                            ));
                        }
                    }
                    _ => {
                        return Err(StdError::generic_err("Wrong execution call"));
                    }
                }

                Err(StdError::generic_err("Wrong execution call"))
            },
            |_, _, _, _: Empty| Ok(Response::new()),
            |_, _, _: Empty| to_json_binary(&{}),
        );
    Box::new(contract)
}

fn get_swap_operation(operation: SwapOperation) -> StdResult<String> {
    match operation {
        SwapOperation::NativeSwap {
            offer_denom,
            ask_denom,
        } => Ok(format!("{}/{}", offer_denom, ask_denom)),
        _ => Err(StdError::generic_err("Wrong token type")),
    }
}

fn instantiate_router_contract(app: &mut App) -> Addr {
    instantiate_contract(
        app,
        router_contract,
        "astroport router contract".to_string(),
    )
}

fn astroport_handler_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn instantiate_astroport_handler_contract(app: &mut App, id: u64, msg: InstantiateMsg) -> Addr {
    app.instantiate_contract(
        id,
        Addr::unchecked("sender"),
        &msg,
        &[],
        "astroport exchange hanlder contract",
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
        owner: OWNER_CONTRACT_ADDR.to_string(),
        core_contract: CORE_CONTRACT_ADDR.to_string(),
        cron_address: CRON_ADDR.to_string(),
        pair_contract: "pair_contract".to_string(),
        router_contract: "router_contract".to_string(),
        from_denom: "ueth".to_string(),
        min_rewards: Uint128::one(),
    };

    let info = mock_info(OWNER_CONTRACT_ADDR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    assert_eq!(
        res.events,
        vec![Event::new(
            "crates.io:drop-staking__drop-astroport-exchange-handler-instantiate".to_string()
        )
        .add_attributes(vec![
            Attribute::new("owner".to_string(), OWNER_CONTRACT_ADDR.to_string()),
            Attribute::new("core_contract".to_string(), CORE_CONTRACT_ADDR.to_string()),
            Attribute::new("cron_address".to_string(), CRON_ADDR.to_string()),
            Attribute::new("pair_contract".to_string(), "pair_contract".to_string()),
            Attribute::new("router_contract".to_string(), "router_contract".to_string()),
            Attribute::new("from_denom".to_string(), "ueth".to_string()),
            Attribute::new("min_rewards".to_string(), Uint128::one()),
        ])]
    );
}

#[test]
fn test_config_query() {
    let mut app = mock_app();

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
        },
    );

    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(astroport_handler_contract.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        ConfigResponse {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
            swap_operations: None,
        }
    );
}

#[test]
fn test_exchange_through_pair_call() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let pair_contract = instantiate_pair_contract(&mut app);
    let router_contract = instantiate_router_contract(&mut app);

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: pair_contract.to_string(),
            router_contract: router_contract.to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
        },
    );

    let amount = coins(100, "ueth");
    let _ = app
        .send_tokens(sender_address, astroport_handler_contract.clone(), &amount)
        .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            astroport_handler_contract.clone(),
            &ExecuteMsg::Exchange {},
            &[],
        )
        .unwrap();

    let ty = res.events[4].ty.clone();

    assert_eq!(ty, "wasm-pair_mock-pair_contract_execute".to_string());

    let attrs = res.events[4].attributes[1..].to_vec();

    assert_eq!(
        attrs,
        vec![
            Attribute::new("message".to_string(), "PairExecuteMsg::Swap".to_string()),
            Attribute::new("to".to_string(), CORE_CONTRACT_ADDR.to_string()),
            Attribute::new("ask_asset_info".to_string(), "false".to_string()),
            Attribute::new("belief_price".to_string(), "false".to_string()),
            Attribute::new("max_spread".to_string(), "false".to_string()),
            Attribute::new("offer_asset".to_string(), "100ueth".to_string()),
            Attribute::new("funds_received".to_string(), "100ueth".to_string()),
        ]
    );
}

#[test]
fn test_exchange_through_router_call() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let pair_contract = instantiate_pair_contract(&mut app);
    let router_contract = instantiate_router_contract(&mut app);

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: pair_contract.to_string(),
            router_contract: router_contract.to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
        },
    );

    let amount = coins(100, "ueth");
    let _ = app
        .send_tokens(sender_address, astroport_handler_contract.clone(), &amount)
        .unwrap();

    let _res = app
        .execute_contract(
            Addr::unchecked(OWNER_CONTRACT_ADDR),
            astroport_handler_contract.clone(),
            &ExecuteMsg::UpdateSwapOperations {
                operations: Some(vec![
                    SwapOperation::NativeSwap {
                        offer_denom: "ueth".to_string(),
                        ask_denom: "untrn".to_string(),
                    },
                    SwapOperation::NativeSwap {
                        offer_denom: "untrn".to_string(),
                        ask_denom: "ueth".to_string(),
                    },
                ]),
            },
            &[],
        )
        .unwrap();

    let res = app
        .execute_contract(
            Addr::unchecked(CORE_CONTRACT_ADDR),
            astroport_handler_contract.clone(),
            &ExecuteMsg::Exchange {},
            &[],
        )
        .unwrap();

    let ty = res.events[4].ty.clone();

    assert_eq!(ty, "wasm-router_mock-router_contract_execute".to_string());

    let attrs = res.events[4].attributes[1..].to_vec();

    assert_eq!(
        attrs,
        vec![
            Attribute::new(
                "message".to_string(),
                "RouterExecuteMsg::ExecuteSwapOperations".to_string()
            ),
            Attribute::new("to".to_string(), CORE_CONTRACT_ADDR.to_string()),
            Attribute::new("minimum_receive".to_string(), "false".to_string()),
            Attribute::new("max_spread".to_string(), "false".to_string()),
            Attribute::new("funds_received".to_string(), "100ueth".to_string()),
            Attribute::new("operation1".to_string(), "ueth/untrn".to_string()),
            Attribute::new("operation2".to_string(), "untrn/ueth".to_string()),
        ]
    );
}

#[test]
fn test_not_enough_balance_error() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::from(200u128),
        },
    );

    let amount = coins(100, "ueth");
    let _ = app
        .send_tokens(sender_address, astroport_handler_contract.clone(), &amount)
        .unwrap();

    let res = app.execute_contract(
        Addr::unchecked(CORE_CONTRACT_ADDR),
        astroport_handler_contract.clone(),
        &ExecuteMsg::Exchange {},
        &[],
    );

    let unwrapped_err = res.unwrap_err();
    let chain: Vec<_> = unwrapped_err.chain().collect();
    assert_eq!(
        chain[1].to_string(),
        "Low balance to perform swap operation. Minimum: 200ueth, current: 100ueth",
    );
}

#[test]
fn test_unauthorized_router_call() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
        },
    );

    let result = app.execute_contract(
        sender_address,
        astroport_handler_contract.clone(),
        &ExecuteMsg::UpdateSwapOperations {
            operations: Some(vec![
                SwapOperation::NativeSwap {
                    offer_denom: "ueth".to_string(),
                    ask_denom: "untrn".to_string(),
                },
                SwapOperation::NativeSwap {
                    offer_denom: "untrn".to_string(),
                    ask_denom: "ueth".to_string(),
                },
            ]),
        },
        &[],
    );

    assert!(result.is_err());

    let unwrapped_err = result.unwrap_err();
    let chain: Vec<_> = unwrapped_err.chain().collect();
    assert_eq!(
        chain[1].to_string(),
        "Caller is not the contract's current owner",
    );
}

#[test]
fn test_unauthorized_config_update() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
        },
    );

    let result = app.execute_contract(
        sender_address,
        astroport_handler_contract.clone(),
        &ExecuteMsg::UpdateConfig {
            owner: Some(OWNER_CONTRACT_ADDR.to_string()),
            core_contract: Some(CORE_CONTRACT_ADDR.to_string()),
            cron_address: Some(CRON_ADDR.to_string()),
            pair_contract: Some("pair_contract".to_string()),
            router_contract: Some("router_contract".to_string()),
            from_denom: Some("ueth".to_string()),
            min_rewards: Some(Uint128::one()),
        },
        &[],
    );

    assert!(result.is_err());

    let unwrapped_err = result.unwrap_err();
    let chain: Vec<_> = unwrapped_err.chain().collect();
    assert_eq!(
        chain[1].to_string(),
        "Caller is not the contract's current owner",
    );
}

#[test]
fn test_config_update() {
    let mut app = mock_app();

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
        },
    );

    let res = app
        .execute_contract(
            Addr::unchecked(OWNER_CONTRACT_ADDR),
            astroport_handler_contract.clone(),
            &ExecuteMsg::UpdateConfig {
                owner: Some("owner1".to_string()),
                core_contract: Some("core1".to_string()),
                cron_address: Some("cron1".to_string()),
                pair_contract: Some("pair_contract_1".to_string()),
                router_contract: Some("router_contract_1".to_string()),
                from_denom: Some("untrn".to_string()),
                min_rewards: Some(Uint128::zero()),
            },
            &[],
        )
        .unwrap();

    let ty = res.events[1].ty.clone();

    assert_eq!(
        ty,
        "wasm-crates.io:drop-staking__drop-astroport-exchange-handler-config_update".to_string()
    );

    let attrs = res.events[1].attributes[1..].to_vec();

    assert_eq!(
        attrs,
        vec![
            Attribute::new("owner".to_string(), "owner1".to_string()),
            Attribute::new("core_contract".to_string(), "core1".to_string()),
            Attribute::new("cron_address".to_string(), "cron1".to_string()),
            Attribute::new(
                "router_contract".to_string(),
                "router_contract_1".to_string()
            ),
            Attribute::new("pair_contract".to_string(), "pair_contract_1".to_string()),
            Attribute::new("from_denom".to_string(), "untrn".to_string()),
            Attribute::new("min_rewards".to_string(), Uint128::zero()),
        ]
    );

    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(astroport_handler_contract.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        ConfigResponse {
            owner: "owner1".to_string(),
            core_contract: "core1".to_string(),
            cron_address: "cron1".to_string(),
            pair_contract: "pair_contract_1".to_string(),
            router_contract: "router_contract_1".to_string(),
            from_denom: "untrn".to_string(),
            min_rewards: Uint128::zero(),
            swap_operations: None,
        }
    );
}

#[test]
fn test_swap_operations_update() {
    let mut app = mock_app();

    let sender_address = Addr::unchecked(SENDER_ADDR);

    let astroport_exchange_handler_code_id = app.store_code(astroport_handler_contract());

    let astroport_handler_contract = instantiate_astroport_handler_contract(
        &mut app,
        astroport_exchange_handler_code_id,
        InstantiateMsg {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
        },
    );

    let amount = coins(100, "ueth");
    let _ = app
        .send_tokens(sender_address, astroport_handler_contract.clone(), &amount)
        .unwrap();

    let operations = vec![
        SwapOperation::NativeSwap {
            offer_denom: "ueth".to_string(),
            ask_denom: "untrn".to_string(),
        },
        SwapOperation::NativeSwap {
            offer_denom: "untrn".to_string(),
            ask_denom: "ueth".to_string(),
        },
    ];
    let _res = app
        .execute_contract(
            Addr::unchecked(OWNER_CONTRACT_ADDR),
            astroport_handler_contract.clone(),
            &ExecuteMsg::UpdateSwapOperations {
                operations: Some(operations.clone()),
            },
            &[],
        )
        .unwrap();

    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(astroport_handler_contract.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        ConfigResponse {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
            swap_operations: Some(operations),
        }
    );

    let _res = app
        .execute_contract(
            Addr::unchecked(OWNER_CONTRACT_ADDR),
            astroport_handler_contract.clone(),
            &ExecuteMsg::UpdateSwapOperations { operations: None },
            &[],
        )
        .unwrap();

    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(astroport_handler_contract.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        ConfigResponse {
            owner: OWNER_CONTRACT_ADDR.to_string(),
            core_contract: CORE_CONTRACT_ADDR.to_string(),
            cron_address: CRON_ADDR.to_string(),
            pair_contract: "pair_contract".to_string(),
            router_contract: "router_contract".to_string(),
            from_denom: "ueth".to_string(),
            min_rewards: Uint128::one(),
            swap_operations: None,
        }
    );
}
