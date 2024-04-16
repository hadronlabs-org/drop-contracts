use crate::{
    contract::{self, CREATE_DENOM_REPLY_ID},
    error::ContractError,
};
use cosmos_sdk_proto::{
    cosmos::bank::v1beta1::{DenomUnit, Metadata},
    prost::Message,
};
use cosmwasm_std::{
    attr, coin,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    to_json_binary, Addr, Binary, CosmosMsg, Event, QueryRequest, Reply, ReplyOn, SubMsgResult,
    Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::{
    msg::token::{ConfigResponse, DenomMetadata, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::token::{CORE_ADDRESS, DENOM, TOKEN_METADATA},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    proto_types::osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata,
    query::token_factory::FullDenomResponse,
};

fn sample_metadata() -> DenomMetadata {
    DenomMetadata {
        exponent: 6,
        display: "token".to_string(),
        name: "A token".to_string(),
        description: "Some token used for testing".to_string(),
        symbol: "TOKEN".to_string(),
        uri: None,
        uri_hash: None,
    }
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        InstantiateMsg {
            core_address: "core".to_string(),
            subdenom: "subdenom".to_string(),
            token_metadata: sample_metadata(),
            owner: "admin".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        CORE_ADDRESS.load(deps.as_ref().storage).unwrap(),
        Addr::unchecked("core")
    );
    assert_eq!(
        TOKEN_METADATA.load(deps.as_ref().storage).unwrap(),
        sample_metadata(),
    );

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
        vec![Event::new("drop-token-instantiate")
            .add_attributes([attr("core_address", "core"), attr("subdenom", "subdenom")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn reply_unknown_id() {
    let mut deps = mock_dependencies(&[]);
    let error = contract::reply(
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
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, "subdenom");
                to_json_binary(&FullDenomResponse {
                    denom: "factory/subdenom".to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });
    DENOM
        .save(deps.as_mut().storage, &String::from("subdenom"))
        .unwrap();
    TOKEN_METADATA
        .save(deps.as_mut().storage, &sample_metadata())
        .unwrap();

    let response = contract::reply(
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

    assert_eq!(response.messages.len(), 1);
    match response.messages[0].msg.clone() {
        CosmosMsg::Stargate { type_url, value } => {
            assert_eq!(
                type_url,
                "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata"
            );
            assert_eq!(
                value,
                Binary::from(
                    MsgSetDenomMetadata {
                        sender: MOCK_CONTRACT_ADDR.to_string(),
                        metadata: Some(Metadata {
                            description: "Some token used for testing".to_string(),
                            denom_units: vec![
                                DenomUnit {
                                    denom: denom.clone(),
                                    exponent: 0,
                                    aliases: vec![],
                                },
                                DenomUnit {
                                    denom: "token".to_string(),
                                    exponent: 6,
                                    aliases: vec![],
                                },
                            ],
                            base: denom,
                            display: "token".to_string(),
                            name: "A token".to_string(),
                            symbol: "TOKEN".to_string(),
                            uri: "".to_string(),
                            uri_hash: "".to_string(),
                        })
                    }
                    .encode_to_vec()
                )
            );
        }
        _ => panic!(),
    };
    assert_eq!(
        response.events,
        vec![Event::new("drop-token-reply-create-denom")
            .add_attributes([attr("denom", "factory/subdenom")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn mint_zero() {
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = contract::execute(
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
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let response = contract::execute(
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
        vec![Event::new("drop-token-execute-mint")
            .add_attributes([attr("amount", "220denom"), attr("receiver", "receiver")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn mint_stranger() {
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = contract::execute(
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
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = contract::execute(
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
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = contract::execute(
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
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = contract::execute(
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
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let response = contract::execute(
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
        vec![Event::new("drop-token-execute-burn").add_attributes([attr("amount", "140denom")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn burn_stranger() {
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let error = contract::execute(
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
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &String::from("denom"))
        .unwrap();

    let response = contract::query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    assert_eq!(
        response,
        to_json_binary(&ConfigResponse {
            core_address: "core".to_string(),
            denom: "denom".to_string()
        })
        .unwrap()
    );
}
