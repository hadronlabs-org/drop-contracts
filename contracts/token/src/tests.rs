use cosmwasm_std::{
    attr, coin, from_json,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
    to_json_binary, Addr, ContractResult, CosmosMsg, Event, OwnedDeps, Querier, QuerierResult,
    QueryRequest, Reply, ReplyOn, SubMsgResult, SystemError, Uint128,
};
use lido_staking_base::{
    msg::token::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::token::{CORE_ADDRESS, DENOM},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::FullDenomResponse,
};
use std::marker::PhantomData;

use crate::{
    contract::{self, CREATE_DENOM_REPLY_ID},
    error::ContractError,
};

fn mock_dependencies<Q: Querier + Default>() -> OwnedDeps<MockStorage, MockApi, Q, NeutronQuery> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    }
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies::<MockQuerier>();
    let response = contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        InstantiateMsg {
            core_address: "core".to_string(),
            subdenom: "subdenom".to_string(),
        },
    )
    .unwrap();

    let core = CORE_ADDRESS.load(deps.as_ref().storage).unwrap();
    assert_eq!(core, Addr::unchecked("core"));

    let denom = DENOM.load(deps.as_ref().storage).unwrap();
    assert_eq!(denom, "subdenom");

    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].reply_on, ReplyOn::Success);
    assert_eq!(response.messages[0].id, CREATE_DENOM_REPLY_ID);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Custom(NeutronMsg::CreateDenom {
            subdenom: "subdenom".to_string()
        })
    );
    assert_eq!(
        response.events,
        vec![Event::new("lido-token-instantiate")
            .add_attributes([attr("core_address", core), attr("subdenom", "subdenom")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn reply_unknown_id() {
    let mut deps = mock_dependencies::<MockQuerier>();
    let error = crate::contract::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 215,
            result: SubMsgResult::Err("".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(error, ContractError::UnknownReplyId { id: 215 });
}

#[test]
fn reply() {
    #[derive(Default)]
    struct CustomMockQuerier {}
    impl Querier for CustomMockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request =
                match from_json::<QueryRequest<NeutronQuery>>(bin_request).map_err(move |err| {
                    QuerierResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", err),
                        request: bin_request.into(),
                    })
                }) {
                    Ok(v) => v,
                    Err(e) => return e,
                };
            match request {
                QueryRequest::Custom(request) => match request {
                    NeutronQuery::FullDenom {
                        creator_addr,
                        subdenom,
                    } => {
                        assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                        assert_eq!(subdenom, "subdenom");
                        QuerierResult::Ok(ContractResult::Ok(
                            to_json_binary(&FullDenomResponse {
                                denom: "factory/subdenom".to_string(),
                            })
                            .unwrap(),
                        ))
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    }

    let mut deps = mock_dependencies::<CustomMockQuerier>();
    DENOM
        .save(deps.as_mut().storage, &String::from("subdenom"))
        .unwrap();
    let response = crate::contract::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: CREATE_DENOM_REPLY_ID,
            result: SubMsgResult::Err("".to_string()),
        },
    )
    .unwrap();

    let denom = DENOM.load(deps.as_ref().storage).unwrap();
    assert_eq!(denom, "factory/subdenom");

    assert!(response.messages.is_empty());
    assert_eq!(
        response.events,
        vec![Event::new("lido-token-reply-create-denom")
            .add_attributes([attr("denom", "factory/subdenom")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn mint_zero() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        ExecuteMsg::Mint {
            amount: Uint128::zero(),
            receiver: "receiver".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(error, ContractError::NothingToMint);
}

#[test]
fn mint() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        ExecuteMsg::Mint {
            amount: Uint128::new(220),
            receiver: "receiver".to_string(),
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Custom(NeutronMsg::MintTokens {
            denom: "denom".to_string(),
            amount: Uint128::new(220),
            mint_to_address: "receiver".to_string(),
        })
    );
    assert_eq!(
        response.events,
        vec![Event::new("lido-token-execute-mint")
            .add_attributes([attr("amount", "220denom"), attr("receiver", "receiver")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn mint_stranger() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stranger", &[]),
        ExecuteMsg::Mint {
            amount: Uint128::new(220),
            receiver: "receiver".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(error, ContractError::Unauthorized);
}

#[test]
fn burn_zero() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        ExecuteMsg::Burn {},
    )
    .unwrap_err();
    assert_eq!(
        error,
        ContractError::PaymentError(cw_utils::PaymentError::NoFunds {})
    );
}

#[test]
fn burn_multiple_coins() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[coin(20, "coin1"), coin(10, "denom")]),
        ExecuteMsg::Burn {},
    )
    .unwrap_err();
    assert_eq!(
        error,
        ContractError::PaymentError(cw_utils::PaymentError::MultipleDenoms {})
    );
}

#[test]
fn burn_invalid_coin() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[coin(20, "coin1")]),
        ExecuteMsg::Burn {},
    )
    .unwrap_err();
    assert_eq!(
        error,
        ContractError::PaymentError(cw_utils::PaymentError::MissingDenom("denom".to_string()))
    );
}

#[test]
fn burn() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[coin(140, "denom")]),
        ExecuteMsg::Burn {},
    )
    .unwrap();

    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Custom(NeutronMsg::BurnTokens {
            denom: "denom".to_string(),
            amount: Uint128::new(140),
            burn_from_address: "".to_string(),
        })
    );
    assert_eq!(
        response.events,
        vec![Event::new("lido-token-execute-burn").add_attributes([attr("amount", "140denom")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn burn_stranger() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stranger", &[coin(160, "denom")]),
        ExecuteMsg::Burn {},
    )
    .unwrap_err();

    assert_eq!(error, ContractError::Unauthorized);
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies::<MockQuerier>();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let response = crate::contract::query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    assert_eq!(
        response,
        to_json_binary(&ConfigResponse {
            core_address: "core".to_string(),
            denom: "denom".to_string()
        })
        .unwrap()
    );
}
