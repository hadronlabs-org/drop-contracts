use crate::msg::{BondingsResponse, QueryMsg};
use crate::store::reply::{CoreUnbond, CORE_UNBOND};
use crate::{
    contract::{self, CORE_UNBOND_REPLY_ID},
    error::ContractError,
    msg::{BondMsg, ExecuteMsg, InstantiateMsg},
    store::{
        CORE_ADDRESS, LD_TOKEN, WITHDRAWAL_DENOM_PREFIX, WITHDRAWAL_MANAGER_ADDRESS,
        WITHDRAWAL_TOKEN_ADDRESS,
    },
};
use cosmwasm_std::{
    attr, coin, from_json,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Event, OwnedDeps, Querier, Reply, ReplyOn,
    Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128, Uint64, WasmMsg,
};
use neutron_sdk::bindings::query::NeutronQuery;
use std::marker::PhantomData;

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
            withdrawal_token_address: "withdrawal_token".to_string(),
            withdrawal_manager_address: "withdrawal_manager".to_string(),
            ld_token: "ld_token".to_string(),
            withdrawal_denom_prefix: "drop".to_string(),
        },
    )
    .unwrap();

    let core = CORE_ADDRESS.load(deps.as_ref().storage).unwrap();
    assert_eq!(core, "core");
    let ld_token = LD_TOKEN.load(deps.as_ref().storage).unwrap();
    assert_eq!(ld_token, "ld_token");
    let withdrawal_token = WITHDRAWAL_TOKEN_ADDRESS
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(withdrawal_token, "withdrawal_token");
    let withdrawal_manager = WITHDRAWAL_MANAGER_ADDRESS
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(withdrawal_manager, "withdrawal_manager");
    let withdrawal_denom_prefix = WITHDRAWAL_DENOM_PREFIX.load(deps.as_ref().storage).unwrap();
    assert_eq!(withdrawal_denom_prefix, "drop");

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("drop-auto-withdrawer-instantiate").add_attributes([
                attr("core_address", "core"),
                attr("withdrawal_token", "withdrawal_token"),
                attr("withdrawal_manager", "withdrawal_manager"),
                attr("ld_token", "ld_token"),
                attr("withdrawal_denom_prefix", "drop")
            ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn bond_missing_ld_assets() {
    let mut deps = mock_dependencies::<MockQuerier>();
    LD_TOKEN
        .save(deps.as_mut().storage, &"ld_token".into())
        .unwrap();
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[coin(10, "uatom"), coin(20, "untrn")]),
        ExecuteMsg::Bond(BondMsg::WithLdAssets {}),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::LdTokenExpected {});
}

#[test]
fn bond_missing_withdrawal_denoms() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::WithdrawalAssetExpected {});
}

mod bond_missing_deposit {
    use super::*;

    #[test]
    fn with_ld_assets() {
        let mut deps = mock_dependencies::<MockQuerier>();
        LD_TOKEN
            .save(deps.as_mut().storage, &"ld_token".into())
            .unwrap();
        let err = contract::execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[coin(10, "ld_token")]),
            ExecuteMsg::Bond(BondMsg::WithLdAssets {}),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::DepositExpected {});
    }

    #[test]
    fn with_nft() {
        let mut deps = mock_dependencies::<MockQuerier>();

        WITHDRAWAL_DENOM_PREFIX
            .save(deps.as_mut().storage, &"drop".into())
            .unwrap();
        WITHDRAWAL_TOKEN_ADDRESS
            .save(
                deps.as_mut().storage,
                &Addr::unchecked("withdrawal_token_contract"),
            )
            .unwrap();

        let err = contract::execute(
            deps.as_mut(),
            mock_env(),
            mock_info(
                "sender",
                &[coin(10, "factory/withdrawal_token_contract/drop:unbond:0")],
            ),
            ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
                batch_id: Uint128::zero(),
            }),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::DepositExpected {});
    }
}

#[test]
fn bond_with_ld_assets_happy_path() {
    let mut deps = mock_dependencies::<MockQuerier>();

    LD_TOKEN
        .save(deps.as_mut().storage, &"ld_token".into())
        .unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[coin(10, "ld_token"), coin(20, "untrn")]),
        ExecuteMsg::Bond(BondMsg::WithLdAssets {}),
    )
    .unwrap();

    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].reply_on, ReplyOn::Success);
    assert_eq!(response.messages[0].id, CORE_UNBOND_REPLY_ID);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "core_contract".to_string(),
            msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Unbond {}).unwrap(),
            funds: vec![Coin::new(10, "ld_token")],
        })
    );
    assert!(response.events.is_empty());
    assert!(response.attributes.is_empty());
}

#[test]
fn reply_after_new_bond_with_ld_assets() {
    let mut deps = drop_helpers::testing::mock_dependencies(&[]);

    CORE_UNBOND
        .save(
            deps.as_mut().storage,
            &CoreUnbond {
                sender: Addr::unchecked("sender"),
                deposit: vec![Coin::new(10, "untrn")],
            },
        )
        .unwrap();

    let response = contract::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: CORE_UNBOND_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm")
                    .add_attribute("denom", "factory/withdrawal_token_contract/drop:unbond:0")
                    .add_attribute("receiver", "receiver")
                    .add_attribute("batch_id", "0")
                    .add_attribute("amount", "100")],
                data: None,
            }),
        },
    )
    .unwrap();

    assert_eq!(response, Response::new());

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Bondings {
            user: None,
            limit: Option::from(Uint64::new(10u64)),
            page_key: None,
        },
    );

    let bondings_response = from_json::<BondingsResponse>(res.unwrap()).unwrap();
    assert_eq!(bondings_response.bondings.len(), 1);

    assert_eq!(bondings_response.bondings[0].bonding_id, "sender_0");
    assert_eq!(bondings_response.bondings[0].bonder, "sender");

    assert_eq!(
        bondings_response.bondings[0].withdrawal_amount,
        Uint128::from(100u64)
    );

    assert_eq!(bondings_response.bondings[0].deposit.len(), 1);
    assert_eq!(
        bondings_response.bondings[0].deposit[0].denom,
        "untrn".to_string()
    );
    assert_eq!(
        bondings_response.bondings[0].deposit[0].amount,
        Uint128::from(10u64)
    );
}

#[test]
fn reply_after_existing_bond_with_ld_assets() {
    let mut deps = drop_helpers::testing::mock_dependencies(&[]);

    CORE_UNBOND
        .save(
            deps.as_mut().storage,
            &CoreUnbond {
                sender: Addr::unchecked("sender"),
                deposit: vec![Coin::new(8, "untrn")],
            },
        )
        .unwrap();

    let _ = contract::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: CORE_UNBOND_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm")
                    .add_attribute("denom", "factory/withdrawal_token_contract/drop:unbond:0")
                    .add_attribute("receiver", "receiver")
                    .add_attribute("batch_id", "0")
                    .add_attribute("amount", "80")],
                data: None,
            }),
        },
    );

    CORE_UNBOND
        .save(
            deps.as_mut().storage,
            &CoreUnbond {
                sender: Addr::unchecked("sender"),
                deposit: vec![Coin::new(12, "untrn")],
            },
        )
        .unwrap();

    let response = contract::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: CORE_UNBOND_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm")
                    .add_attribute("denom", "factory/withdrawal_token_contract/drop:unbond:0")
                    .add_attribute("receiver", "receiver")
                    .add_attribute("batch_id", "0")
                    .add_attribute("amount", "120")],
                data: None,
            }),
        },
    )
    .unwrap();

    assert_eq!(response, Response::new());

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Bondings {
            user: None,
            limit: Option::from(Uint64::new(10u64)),
            page_key: None,
        },
    );

    let bondings_response = from_json::<BondingsResponse>(res.unwrap()).unwrap();
    assert_eq!(bondings_response.bondings.len(), 1);

    assert_eq!(bondings_response.bondings[0].bonding_id, "sender_0");
    assert_eq!(bondings_response.bondings[0].bonder, "sender");

    assert_eq!(
        bondings_response.bondings[0].withdrawal_amount,
        Uint128::from(200u64)
    );

    assert_eq!(bondings_response.bondings[0].deposit.len(), 1);
    assert_eq!(
        bondings_response.bondings[0].deposit[0].denom,
        "untrn".to_string()
    );
    assert_eq!(
        bondings_response.bondings[0].deposit[0].amount,
        Uint128::from(20u64)
    );
}

#[test]
fn reply_unknown_id() {
    let mut deps = drop_helpers::testing::mock_dependencies(&[]);
    let error = contract::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 512,
            result: SubMsgResult::Err("".to_string()),
        },
    )
    .unwrap_err();
    assert_eq!(error, ContractError::InvalidCoreReplyId { id: 512 });
}

#[test]
fn reply_invalid_attribute() {
    let mut deps = drop_helpers::testing::mock_dependencies(&[]);

    CORE_UNBOND
        .save(
            deps.as_mut().storage,
            &CoreUnbond {
                sender: Addr::unchecked("sender"),
                deposit: vec![Coin::new(10, "untrn")],
            },
        )
        .unwrap();

    let error = contract::reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: CORE_UNBOND_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm")
                    .add_attribute("denom", "factory/withdrawal_token_contract/drop:unbond:0")
                    .add_attribute("receiver", "receiver")
                    .add_attribute("batch_id", "0")
                    .add_attribute("amount", "invalid")],
                data: None,
            }),
        },
    )
    .unwrap_err();

    assert_eq!(error, ContractError::InvalidCoreReplyAttributes {});
}

#[test]
fn bond_with_withdrawal_denoms_for_new_bond() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[
                coin(100, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(10, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    assert_eq!(response, Response::new());

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Bondings {
            user: None,
            limit: Option::from(Uint64::new(10u64)),
            page_key: None,
        },
    );

    let bondings_response = from_json::<BondingsResponse>(res.unwrap()).unwrap();
    assert_eq!(bondings_response.bondings.len(), 1);

    assert_eq!(bondings_response.bondings[0].bonding_id, "sender_0");
    assert_eq!(bondings_response.bondings[0].bonder, "sender");

    assert_eq!(
        bondings_response.bondings[0].withdrawal_amount,
        Uint128::from(100u64)
    );

    assert_eq!(bondings_response.bondings[0].deposit.len(), 1);
    assert_eq!(
        bondings_response.bondings[0].deposit[0].denom,
        "untrn".to_string()
    );
    assert_eq!(
        bondings_response.bondings[0].deposit[0].amount,
        Uint128::from(10u64)
    );
}

#[test]
fn bond_with_withdrawal_denoms_for_existing_bond() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[
                coin(120, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(12, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[
                coin(80, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(8, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    assert_eq!(response, Response::new());

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Bondings {
            user: None,
            limit: Option::from(Uint64::new(10u64)),
            page_key: None,
        },
    );

    let bondings_response = from_json::<BondingsResponse>(res.unwrap()).unwrap();
    assert_eq!(bondings_response.bondings.len(), 1);

    assert_eq!(bondings_response.bondings[0].bonding_id, "sender_0");
    assert_eq!(bondings_response.bondings[0].bonder, "sender");

    assert_eq!(
        bondings_response.bondings[0].withdrawal_amount,
        Uint128::from(200u64)
    );

    assert_eq!(bondings_response.bondings[0].deposit.len(), 1);
    assert_eq!(
        bondings_response.bondings[0].deposit[0].denom,
        "untrn".to_string()
    );
    assert_eq!(
        bondings_response.bondings[0].deposit[0].amount,
        Uint128::from(20u64)
    );
}

#[test]
fn second_bond_with_withdrawal_denoms() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let first_response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "first_sender",
            &[
                coin(30, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(3, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    assert_eq!(first_response, Response::new());

    let second_response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "second_sender",
            &[
                coin(60, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(6, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    assert_eq!(second_response, Response::new());

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Bondings {
            user: None,
            limit: Option::from(Uint64::new(10u64)),
            page_key: None,
        },
    );

    let bondings_response = from_json::<BondingsResponse>(res.unwrap()).unwrap();
    assert_eq!(bondings_response.bondings.len(), 2);

    assert_eq!(bondings_response.bondings[0].bonding_id, "first_sender_0");
    assert_eq!(bondings_response.bondings[0].bonder, "first_sender");

    assert_eq!(bondings_response.bondings[1].bonding_id, "second_sender_0");
    assert_eq!(bondings_response.bondings[1].bonder, "second_sender");

    assert_eq!(
        bondings_response.bondings[0].withdrawal_amount,
        Uint128::from(30u64)
    );
    assert_eq!(
        bondings_response.bondings[1].withdrawal_amount,
        Uint128::from(60u64)
    );

    assert_eq!(bondings_response.bondings[0].deposit.len(), 1);
    assert_eq!(
        bondings_response.bondings[0].deposit[0].denom,
        "untrn".to_string()
    );
    assert_eq!(
        bondings_response.bondings[0].deposit[0].amount,
        Uint128::from(3u64)
    );

    assert_eq!(bondings_response.bondings[1].deposit.len(), 1);
    assert_eq!(
        bondings_response.bondings[1].deposit[0].denom,
        "untrn".to_string()
    );
    assert_eq!(
        bondings_response.bondings[1].deposit[0].amount,
        Uint128::from(6u64)
    );
}

#[test]
fn unbond_happy_path() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[
                coin(100, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(10, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Unbond {
            batch_id: Uint128::zero(),
        },
    )
    .unwrap();

    assert_eq!(
        response,
        Response::new()
            .add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "sender".to_string(),
                amount: vec![Coin::new(
                    100,
                    "factory/withdrawal_token_contract/drop:unbond:0"
                )]
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "sender".to_string(),
                amount: vec![Coin::new(10, "untrn")],
            })))
    );
}

#[test]
fn execute_withdraw_nothing() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[
                coin(100, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(10, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    let error = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Withdraw {
            batch_id: Uint128::zero(),
            receiver: None,
            amount: Uint128::zero(),
        },
    )
    .unwrap_err();

    assert_eq!(error, ContractError::NothingToWithdraw {});
}

#[test]
fn execute_withdraw_too_much() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();

    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[
                coin(100, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(10, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    let error = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Withdraw {
            batch_id: Uint128::zero(),
            receiver: None,
            amount: Uint128::from(101u64),
        },
    )
    .unwrap_err();

    assert_eq!(error, ContractError::WithdrawnAmountTooBig {});
}

#[test]
fn execute_withdraw_full_amount() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();

    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "bonder",
            &[
                coin(100, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(10, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    let execute_response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Withdraw {
            batch_id: Uint128::zero(),
            receiver: Option::from(Addr::unchecked("bonder")),
            amount: Uint128::from(100u64),
        },
    )
    .unwrap();

    assert_eq!(
        execute_response,
        Response::new()
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "withdrawal_manager_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::withdrawal_manager::ExecuteMsg::ReceiveWithdrawalDenoms {
                    receiver: Option::from("bonder".to_string()),
                })
                .unwrap(),
                funds: vec![Coin::new(100, "factory/withdrawal_token_contract/drop:unbond:0")],
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "sender".to_string(),
                amount: vec![Coin::new(10, "untrn")],
            })))
    );

    let query_response = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Bondings {
            user: None,
            limit: Option::from(Uint64::new(10u64)),
            page_key: None,
        },
    );

    let bondings_response = from_json::<BondingsResponse>(query_response.unwrap()).unwrap();
    assert_eq!(bondings_response.bondings.len(), 0);
}

#[test]
fn execute_withdraw_part_amount() {
    let mut deps = mock_dependencies::<MockQuerier>();

    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();

    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "bonder",
            &[
                coin(100, "factory/withdrawal_token_contract/drop:unbond:0"),
                coin(10, "untrn"),
            ],
        ),
        ExecuteMsg::Bond(BondMsg::WithWithdrawalDenoms {
            batch_id: Uint128::zero(),
        }),
    )
    .unwrap();

    let execute_response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Withdraw {
            batch_id: Uint128::zero(),
            receiver: Option::from(Addr::unchecked("bonder")),
            amount: Uint128::from(70u64),
        },
    )
    .unwrap();

    assert_eq!(
        execute_response,
        Response::new()
            .add_submessage(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "withdrawal_manager_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::withdrawal_manager::ExecuteMsg::ReceiveWithdrawalDenoms {
                    receiver: Option::from("bonder".to_string()),
                })
                    .unwrap(),
                funds: vec![Coin::new(70, "factory/withdrawal_token_contract/drop:unbond:0")],
            })))
            .add_submessage(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "sender".to_string(),
                amount: vec![Coin::new(7, "untrn")],
            })))
    );

    let query_response = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Bondings {
            user: None,
            limit: Option::from(Uint64::new(10u64)),
            page_key: None,
        },
    );

    let bondings_response = from_json::<BondingsResponse>(query_response.unwrap()).unwrap();
    assert_eq!(bondings_response.bondings.len(), 1);
    assert_eq!(bondings_response.bondings[0].bonding_id, "bonder_0");
    assert_eq!(bondings_response.bondings[0].bonder, "bonder");
    assert_eq!(
        bondings_response.bondings[0].withdrawal_amount,
        Uint128::from(30u64)
    );
    assert_eq!(bondings_response.bondings[0].deposit.len(), 1);
    assert_eq!(
        bondings_response.bondings[0].deposit[0].denom,
        "untrn".to_string()
    );
    assert_eq!(
        bondings_response.bondings[0].deposit[0].amount,
        Uint128::from(3u64)
    );
}

#[test]
fn execute_query_config() {
    let mut deps = mock_dependencies::<MockQuerier>();

    LD_TOKEN
        .save(deps.as_mut().storage, &"ld_token".into())
        .unwrap();
    WITHDRAWAL_DENOM_PREFIX
        .save(deps.as_mut().storage, &"drop".into())
        .unwrap();
    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core_contract"))
        .unwrap();
    WITHDRAWAL_TOKEN_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_token_contract"),
        )
        .unwrap();
    WITHDRAWAL_MANAGER_ADDRESS
        .save(
            deps.as_mut().storage,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();

    let query_response = contract::query(deps.as_ref(), mock_env(), QueryMsg::Config {});

    let config_response = from_json::<InstantiateMsg>(query_response.unwrap()).unwrap();
    assert_eq!(config_response.core_address, "core_contract");
    assert_eq!(config_response.ld_token, "ld_token");
    assert_eq!(config_response.withdrawal_denom_prefix, "drop");
    assert_eq!(
        config_response.withdrawal_token_address,
        "withdrawal_token_contract"
    );
    assert_eq!(
        config_response.withdrawal_manager_address,
        "withdrawal_manager_contract"
    );
}
