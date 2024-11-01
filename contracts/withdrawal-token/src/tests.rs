use crate::contract::{execute, query, reply, CREATE_DENOM_REPLY_ID, UNBOND_MARK};
use cosmwasm_std::{
    attr, coin, from_json,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    to_json_binary, Addr, CosmosMsg, Event, QueryRequest, Reply, ReplyOn, Response, SubMsgResult,
    Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::error::withdrawal_token::ContractError;
use drop_staking_base::msg::withdrawal_token::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use drop_staking_base::state::withdrawal_token::{
    CORE_ADDRESS, DENOM_PREFIX, IS_INIT_STATE, WITHDRAWAL_MANAGER_ADDRESS,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::query::token_factory::FullDenomResponse;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        core_address: "core_contract".to_string(),
        withdrawal_manager_address: "withdrawal_manager_contract".to_string(),
        withdrawal_exchange_address: "withdrawal_exchange_contract".to_string(),
        is_init_state: true,
        denom_prefix: "denom".to_string(),
        owner: "owner".to_string(),
    };
    let env = mock_env();
    let res =
        crate::contract::instantiate(deps.as_mut(), env, mock_info("sender", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("drop-withdrawal-token-instantiate").add_attributes(vec![
                ("core_address", "core_contract"),
                ("withdrawal_manager_address", "withdrawal_manager_contract"),
                (
                    "withdrawal_exchange_address",
                    "withdrawal_exchange_contract"
                ),
            ])
        )
    );
    assert_eq!(
        Addr::unchecked("owner"),
        cw_ownable::get_ownership(deps.as_mut().storage)
            .unwrap()
            .owner
            .unwrap()
    );
}

#[test]
fn test_query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    assert_eq!(
        from_json::<String>(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap())
            .unwrap(),
        String::from("owner"),
    );
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    let response = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    assert_eq!(
        response,
        to_json_binary(&ConfigResponse {
            core_address: "core_contract".to_string(),
            withdrawal_manager_address: "withdrawal_manager_contract".to_string(),
            denom_prefix: "denom_prefix".to_string()
        })
        .unwrap()
    );
}

#[test]
fn test_create_denom() {
    let mut deps = mock_dependencies(&[]);

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    let subdenom = format!("denom_prefix:{}:0", UNBOND_MARK);
    let response = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core_contract", &[]),
        ExecuteMsg::CreateDenom {
            batch_id: Uint128::zero(),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].reply_on, ReplyOn::Success);
    assert_eq!(response.messages[0].id, CREATE_DENOM_REPLY_ID);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Custom(NeutronMsg::CreateDenom {
            subdenom: subdenom.to_string(),
        })
    );
    assert_eq!(
        response.events,
        vec![
            Event::new("drop-withdrawal-token-execute-create-denom").add_attributes([
                attr("batch_id", "0"),
                attr("subdenom", subdenom.to_string())
            ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn reply_unknown_id() {
    let mut deps = mock_dependencies(&[]);
    let error = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 512,
            result: SubMsgResult::Err("".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(error, ContractError::UnknownReplyId { id: 512 });
}

#[test]
fn test_reply() {
    let mut deps = mock_dependencies(&[]);

    let response = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: CREATE_DENOM_REPLY_ID,
            result: SubMsgResult::Err("".to_string()),
        },
    )
    .unwrap();

    assert_eq!(
        response.events,
        vec![Event::new("drop-withdrawal-token-reply-create-denom")
            .add_attributes([attr("denom", "new unbond denom")])]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn test_mint_zero() {
    let mut deps = mock_dependencies(&[]);

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    let error = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        ExecuteMsg::Mint {
            amount: Uint128::zero(),
            receiver: "receiver".to_string(),
            batch_id: Uint128::zero(),
        },
    )
    .unwrap_err();
    assert_eq!(error, ContractError::NothingToMint);
}

#[test]
fn test_mint() {
    let mut deps = mock_dependencies(&[]);

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, &format!("denom_prefix:{}:0", UNBOND_MARK));
                to_json_binary(&FullDenomResponse {
                    denom: format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });

    let response = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core_contract", &[]),
        ExecuteMsg::Mint {
            amount: Uint128::new(228),
            receiver: "receiver".to_string(),
            batch_id: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Custom(NeutronMsg::MintTokens {
            denom: format!(
                "factory/{}/denom_prefix:{}:0",
                MOCK_CONTRACT_ADDR, UNBOND_MARK
            )
            .to_string(),
            amount: Uint128::new(228),
            mint_to_address: "receiver".to_string(),
        })
    );
    assert_eq!(
        response.events,
        vec![
            Event::new("drop-withdrawal-token-execute-mint").add_attributes([
                attr("amount", "228"),
                attr(
                    "denom",
                    format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string()
                ),
                attr("receiver", "receiver"),
                attr("batch_id", "0"),
            ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn mint_stranger() {
    let mut deps = mock_dependencies(&[]);

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    let error = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stranger", &[]),
        ExecuteMsg::Mint {
            amount: Uint128::new(220),
            receiver: "receiver".to_string(),
            batch_id: Uint128::zero(),
        },
    )
    .unwrap_err();

    assert_eq!(error, ContractError::Unauthorized);
}

#[test]
fn burn_zero() {
    let mut deps = mock_dependencies(&[]);

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, &format!("denom_prefix:{}:0", UNBOND_MARK));
                to_json_binary(&FullDenomResponse {
                    denom: format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });

    let error = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("withdrawal_manager_contract", &[]),
        ExecuteMsg::Burn {
            batch_id: Uint128::zero(),
        },
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

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, &format!("denom_prefix:{}:0", UNBOND_MARK));
                to_json_binary(&FullDenomResponse {
                    denom: format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });

    let error = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "withdrawal_manager_contract",
            &[coin(20, "coin1"), coin(10, "denom")],
        ),
        ExecuteMsg::Burn {
            batch_id: Uint128::zero(),
        },
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

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, &format!("denom_prefix:{}:0", UNBOND_MARK));
                to_json_binary(&FullDenomResponse {
                    denom: format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });

    let error = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("withdrawal_manager_contract", &[coin(20, "not_that_coin")]),
        ExecuteMsg::Burn {
            batch_id: Uint128::zero(),
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        ContractError::PaymentError(cw_utils::PaymentError::MissingDenom(
            format!(
                "factory/{}/denom_prefix:{}:0",
                MOCK_CONTRACT_ADDR, UNBOND_MARK
            )
            .to_string()
        ))
    );
}

#[test]
fn burn_stranger() {
    let mut deps = mock_dependencies(&[]);

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, &format!("denom_prefix:{}:0", UNBOND_MARK));
                to_json_binary(&FullDenomResponse {
                    denom: format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });

    let error = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "stranger",
            &[coin(
                160,
                format!(
                    "factory/{}/denom_prefix:{}:0",
                    MOCK_CONTRACT_ADDR, UNBOND_MARK
                )
                .to_string(),
            )],
        ),
        ExecuteMsg::Burn {
            batch_id: Uint128::zero(),
        },
    )
    .unwrap_err();

    assert_eq!(error, ContractError::Unauthorized);
}

#[test]
fn burn() {
    let mut deps = mock_dependencies(&[]);

    IS_INIT_STATE.save(deps.as_mut().storage, &false).unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    DENOM_PREFIX
        .save(deps.as_mut().storage, &String::from("denom_prefix"))
        .unwrap();

    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, &format!("denom_prefix:{}:0", UNBOND_MARK));
                to_json_binary(&FullDenomResponse {
                    denom: format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });

    let response = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "withdrawal_manager_contract",
            &[coin(
                144,
                format!(
                    "factory/{}/denom_prefix:{}:0",
                    MOCK_CONTRACT_ADDR, UNBOND_MARK
                )
                .to_string(),
            )],
        ),
        ExecuteMsg::Burn {
            batch_id: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Custom(NeutronMsg::BurnTokens {
            denom: format!(
                "factory/{}/denom_prefix:{}:0",
                MOCK_CONTRACT_ADDR, UNBOND_MARK
            )
            .to_string(),
            amount: Uint128::new(144),
            burn_from_address: "".to_string(),
        })
    );
    assert_eq!(
        response.events,
        vec![
            Event::new("drop-withdrawal-token-execute-burn").add_attributes([attr(
                "amount",
                format!(
                    "144factory/{}/denom_prefix:{}:0",
                    MOCK_CONTRACT_ADDR, UNBOND_MARK
                )
                .to_string()
            )])
        ]
    );
    assert!(response.attributes.is_empty());
}
