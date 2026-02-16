use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::msg::QueryMsg as ConverterQuery;
use cosmwasm_std::{
    coin, from_json, testing::mock_env, testing::mock_info, Addr, BankMsg, CosmosMsg, Decimal,
    Uint128,
};
use drop_helpers::pause::PauseError;
use drop_helpers::testing::mock_dependencies;

#[test]
fn test_instantiate_and_query() {
    let mut deps = mock_dependencies(&[]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(50),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();

    let info = mock_info("owner", &[]);
    let _res = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    // query config
    let cfg: crate::state::Config = from_json(
        query(
            deps.as_ref().into_empty(),
            env.clone(),
            ConverterQuery::Config {},
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(cfg.from_token, "denom_a");
    assert_eq!(cfg.to_token, "denom_b");

    // query rate
    let rate: Decimal =
        from_json(query(deps.as_ref().into_empty(), env, ConverterQuery::Rate()).unwrap()).unwrap();

    assert_eq!(rate, Decimal::percent(50));
}

#[test]
fn test_update_rate_by_owner() {
    let mut deps = mock_dependencies(&[]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(10),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    // owner updates rate to 25%
    let exec_info = mock_info("owner", &[]);
    let _res = execute(
        deps.as_mut().into_empty(),
        env.clone(),
        exec_info,
        crate::msg::ExecuteMsg::UpdateRate {
            rate: Decimal::percent(25),
        },
    )
    .unwrap();

    let rate: Decimal =
        from_json(query(deps.as_ref().into_empty(), env, ConverterQuery::Rate()).unwrap()).unwrap();

    assert_eq!(rate, Decimal::percent(25));
}

#[test]
fn test_swap_sends_bank_msg_and_attrs() {
    let mut deps = mock_dependencies(&[coin(100u128, "denom_a")]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(50),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    // perform swap: pay 100 denom_a -> expect 50 denom_b to receiver
    let swap_info = cosmwasm_std::MessageInfo {
        sender: Addr::unchecked("some_sender"),
        funds: vec![coin(100u128, "denom_a")],
    };

    let res = execute(
        deps.as_mut().into_empty(),
        env.clone(),
        swap_info,
        crate::msg::ExecuteMsg::Swap {
            receiver: "recipient".to_string(),
        },
    )
    .unwrap();

    // one message expected: BankMsg::Send to recipient with 50 denom_b
    assert_eq!(res.messages.len(), 1);

    match &res.messages[0].msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, "recipient");
            assert_eq!(amount.len(), 1);
            assert_eq!(amount[0].amount, Uint128::from(50u128));
            assert_eq!(amount[0].denom, "denom_b");
        }
        m => panic!("unexpected message: {:?}", m),
    }

    // attributes should include amount_in and amount_out inside events
    let mut amount_in: Option<String> = None;
    let mut amount_out: Option<String> = None;
    for event in &res.events {
        for attr in &event.attributes {
            if attr.key == "amount_in" {
                amount_in = Some(attr.value.clone());
            }
            if attr.key == "amount_out" {
                amount_out = Some(attr.value.clone());
            }
        }
    }

    let amount_in = amount_in.expect("amount_in attribute");
    let amount_out = amount_out.expect("amount_out attribute");

    assert_eq!(amount_in, "100");
    assert_eq!(amount_out, "50");
}

#[test]
fn test_instantiate_same_tokens_error() {
    let mut deps = mock_dependencies(&[]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(50),
        from_token: "denom".to_string(),
        to_token: "denom".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);

    let res = instantiate(deps.as_mut().into_empty(), env, info, init);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::SameTokens {});
}

#[test]
fn test_math_rate_230_percent() {
    let mut deps = mock_dependencies(&[coin(133u128, "denom_a")]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(230),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    let swap_info = cosmwasm_std::MessageInfo {
        sender: Addr::unchecked("some_sender"),
        funds: vec![coin(133u128, "denom_a")],
    };

    let res = execute(
        deps.as_mut().into_empty(),
        env,
        swap_info,
        crate::msg::ExecuteMsg::Swap {
            receiver: "recipient".to_string(),
        },
    )
    .unwrap();

    match &res.messages[0].msg {
        CosmosMsg::Bank(BankMsg::Send { amount, .. }) => {
            // 133 * 2.3 = 305.9 -> expect truncated to 305
            assert_eq!(amount[0].amount, Uint128::from(305u128));
        }
        m => panic!("unexpected message: {:?}", m),
    }
}

#[test]
fn test_math_rate_102_percent() {
    let mut deps = mock_dependencies(&[coin(1000u128, "denom_a")]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(102),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    let swap_info = cosmwasm_std::MessageInfo {
        sender: Addr::unchecked("some_sender"),
        funds: vec![coin(1000u128, "denom_a")],
    };

    let res = execute(
        deps.as_mut().into_empty(),
        env,
        swap_info,
        crate::msg::ExecuteMsg::Swap {
            receiver: "recipient".to_string(),
        },
    )
    .unwrap();

    match &res.messages[0].msg {
        CosmosMsg::Bank(BankMsg::Send { amount, .. }) => {
            assert_eq!(amount[0].amount, Uint128::from(1020u128));
        }
        m => panic!("unexpected message: {:?}", m),
    }
}

#[test]
fn test_clawback_sends_all_funds_to_owner() {
    let mut deps = mock_dependencies(&[coin(10u128, "denom_a"), coin(5u128, "denom_b")]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(100),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    // owner calls clawback
    let exec_info = mock_info("owner", &[]);
    let res = execute(
        deps.as_mut().into_empty(),
        env,
        exec_info,
        crate::msg::ExecuteMsg::Clawback { denom: None },
    )
    .unwrap();

    // expect only the `from_token` (denom_a) to be sent to owner
    assert_eq!(res.messages.len(), 1);

    match &res.messages[0].msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, "owner");
            assert_eq!(amount.len(), 1);
            assert_eq!(amount[0].denom, "denom_a");
            assert_eq!(amount[0].amount, Uint128::from(10u128));
        }
        _ => panic!("unexpected message"),
    }
}

#[test]
fn test_clawback_no_funds_error() {
    let mut deps = mock_dependencies(&[]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(100),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    let exec_info = mock_info("owner", &[]);
    let res = execute(
        deps.as_mut().into_empty(),
        env,
        exec_info,
        crate::msg::ExecuteMsg::Clawback { denom: None },
    );

    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), ContractError::NoFundsToClawback {});
}

#[test]
fn test_pause_blocks_swap_and_unpause_restores() {
    let mut deps = mock_dependencies(&[coin(100u128, "denom_a")]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(100),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    // owner pauses
    let pause_info = mock_info("owner", &[]);
    let _ = execute(
        deps.as_mut().into_empty(),
        env.clone(),
        pause_info,
        crate::msg::ExecuteMsg::Pause {},
    )
    .unwrap();

    // swap should now error with Pause
    let swap_info = cosmwasm_std::MessageInfo {
        sender: Addr::unchecked("some_sender"),
        funds: vec![coin(100u128, "denom_a")],
    };

    let res = execute(
        deps.as_mut().into_empty(),
        env.clone(),
        swap_info,
        crate::msg::ExecuteMsg::Swap {
            receiver: "recipient".to_string(),
        },
    );

    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        ContractError::Pause(PauseError::Paused {})
    );

    // owner unpauses
    let unpause_info = mock_info("owner", &[]);
    let _ = execute(
        deps.as_mut().into_empty(),
        env.clone(),
        unpause_info,
        crate::msg::ExecuteMsg::Unpause {},
    )
    .unwrap();

    // swap should succeed now
    let swap_info2 = cosmwasm_std::MessageInfo {
        sender: Addr::unchecked("some_sender"),
        funds: vec![coin(100u128, "denom_a")],
    };

    let res2 = execute(
        deps.as_mut().into_empty(),
        env,
        swap_info2,
        crate::msg::ExecuteMsg::Swap {
            receiver: "recipient".to_string(),
        },
    )
    .unwrap();

    // ensure a send message was produced
    assert_eq!(res2.messages.len(), 1);
}

#[test]
fn test_clawback_with_specified_denom() {
    let mut deps = mock_dependencies(&[coin(7u128, "denom_a"), coin(42u128, "denom_b")]);

    let init = InstantiateMsg {
        owner: "owner".to_string(),
        rate: Decimal::percent(100),
        from_token: "denom_a".to_string(),
        to_token: "denom_b".to_string(),
    };

    let env = mock_env();
    let info = mock_info("owner", &[]);
    let _ = instantiate(deps.as_mut().into_empty(), env.clone(), info, init).unwrap();

    // owner calls clawback for denom_b explicitly
    let exec_info = mock_info("owner", &[]);
    let res = execute(
        deps.as_mut().into_empty(),
        env,
        exec_info,
        crate::msg::ExecuteMsg::Clawback {
            denom: Some("denom_b".to_string()),
        },
    )
    .unwrap();

    // expect a single send of denom_b (42) to owner
    assert_eq!(res.messages.len(), 1);
    match &res.messages[0].msg {
        CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
            assert_eq!(to_address, "owner");
            assert_eq!(amount.len(), 1);
            assert_eq!(amount[0].denom, "denom_b");
            assert_eq!(amount[0].amount, Uint128::from(42u128));
        }
        _ => panic!("unexpected message"),
    }
}
