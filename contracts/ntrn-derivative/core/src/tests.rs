use crate::{
    contract::{execute, instantiate, query, reply},
    msg::{ExecuteMsg, InstantiateMsg, NftStatus, QueryMsg, ReceiveNftMsg},
    state::{
        Config, BASE_DENOM, CONFIG, CREATE_DENOM_REPLY_ID, DENOM, EXPONENT, SALT, TOKEN_METADATA,
        UNBOND_ID,
    },
};
use cosmos_sdk_proto::cosmos::bank::v1beta1::{DenomUnit, Metadata};
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, DenomMetadata, Event, Reply, ReplyOn,
    Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use cw721::{Cw721ReceiveMsg, NftInfoResponse};
use cw_utils::PaymentError;
use drop_helpers::testing::{mock_dependencies, mock_dependencies_with_api};
use drop_staking_base::{
    msg::ntrn_derivative::withdrawal_voucher::{
        ExecuteMsg as WithdrawalVoucherExecuteMsg, Extension as WithdrawalVoucherExtension,
        InstantiateMsg as WithdrawalVoucherInstantiateMsg,
    },
    state::ntrn_derivative::withdrawal_voucher::Metadata as WithdrawalVoucherMetadata,
};
use neutron_sdk::{
    bindings::msg::NeutronMsg, query::token_factory::FullDenomResponse,
    stargate::aux::create_stargate_msg,
};
use std::sync::Arc;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies_with_api(&[]);
    let msg = InstantiateMsg {
        withdrawal_voucher_code_id: 0,
        unbonding_period: 123,
        token_metadata: DenomMetadata {
            description: "description".to_string(),
            denom_units: vec![],
            base: "base".to_string(),
            display: "display".to_string(),
            name: "name".to_string(),
            symbol: "symbol".to_string(),
            uri: "uri".to_string(),
            uri_hash: "uri_hash".to_string(),
        },
        subdenom: "subdenom".to_string(),
        exponent: 6u32,
    };
    deps.querier.add_stargate_query_response(
        "/cosmos.wasm.v1.Query/QueryCodeRequest",
        |data| -> cosmwasm_std::Binary {
            let mut y = vec![0; 32];
            y[..data.len()].copy_from_slice(data);
            to_json_binary(&cosmwasm_std::CodeInfoResponse::new(
                from_json(data).unwrap(),
                "creator".to_string(),
                cosmwasm_std::HexBinary::from(y.as_slice()),
            ))
            .unwrap()
        },
    );
    let res = instantiate(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        msg,
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    gas_limit: None,
                    reply_on: ReplyOn::Never,
                    msg: CosmosMsg::Wasm(WasmMsg::Instantiate2 {
                        admin: Some("owner".to_string()),
                        code_id: 0,
                        label: "drop-staking-withdrawal-manager".to_string(),
                        msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
                            name: "Drop NTRN Voucher".to_string(),
                            symbol: "DROPV".to_string(),
                            minter: "cosmos2contract".to_string(),
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: Binary::from(SALT.as_bytes())
                    },)
                },
                SubMsg {
                    id: 1,
                    gas_limit: None,
                    reply_on: ReplyOn::Success,
                    msg: CosmosMsg::Custom(NeutronMsg::CreateDenom {
                        subdenom: "subdenom".to_string()
                    }),
                }
            ])
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-instantiate")
                    .add_attributes(vec![
                        attr("owner", "owner"),
                        attr("denom", "subdenom"),
                        attr("withdrawal_voucher_contract", "some_humanized_address")
                    ])
            )
    )
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    let config = Config {
        unbonding_period: 123u64,
        withdrawal_voucher: Addr::unchecked("withdrawal_voucher".to_string()),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res: Config =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(res, config)
}

#[test]
fn test_query_nft_status_ready() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 123u64,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher".to_string()),
            },
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            let resp = &NftInfoResponse::<WithdrawalVoucherExtension> {
                token_uri: Some("token_uri".to_string()),
                extension: Some(WithdrawalVoucherMetadata {
                    name: "name".to_string(),
                    description: Some("description".to_string()),
                    release_at: 0u64,
                    amount: Uint128::from(123u128),
                    recipient: "recipient".to_string(),
                }),
            };
            to_json_binary(resp).unwrap()
        });
    let res: NftStatus = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::NftStatus {
                token_id: "1".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, NftStatus::Ready {})
}

#[test]
fn test_query_nft_status_not_ready() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 123u64,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher".to_string()),
            },
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            let resp = &NftInfoResponse::<WithdrawalVoucherExtension> {
                token_uri: Some("token_uri".to_string()),
                extension: Some(WithdrawalVoucherMetadata {
                    name: "name".to_string(),
                    description: Some("description".to_string()),
                    release_at: u64::MAX,
                    amount: Uint128::from(123u128),
                    recipient: "recipient".to_string(),
                }),
            };
            to_json_binary(resp).unwrap()
        });
    let res: NftStatus = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::NftStatus {
                token_id: "1".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, NftStatus::NotReady {})
}

#[test]
fn test_query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            crate::msg::QueryMsg::Ownership {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(cosmwasm_std::Addr::unchecked("owner".to_string())),
            pending_expiry: None,
            pending_owner: None
        }
    );
}

#[test]
fn test_execute_bond() {
    let mut deps = mock_dependencies(&[]);
    DENOM
        .save(deps.as_mut().storage, &"dNTRN".to_string())
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "some_sender",
            &[Coin {
                denom: BASE_DENOM.to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Bond { receiver: None },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Custom(NeutronMsg::submit_mint_tokens(
                    "dNTRN".to_string(),
                    Uint128::from(100u128),
                    "some_sender".to_string()
                )),
                gas_limit: None,
                reply_on: ReplyOn::Never
            })
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-execute-bond")
                    .add_attributes(vec![
                        attr("action", "bond"),
                        attr("amount", "100"),
                        attr("receiver", "some_sender")
                    ])
            )
    );
}

#[test]
fn test_execute_bond_custom_receiver() {
    let mut deps = mock_dependencies(&[]);
    DENOM
        .save(deps.as_mut().storage, &"dNTRN".to_string())
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "some_sender",
            &[Coin {
                denom: BASE_DENOM.to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Bond {
            receiver: Some("custom_receiver".to_string()),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Custom(NeutronMsg::submit_mint_tokens(
                    "dNTRN".to_string(),
                    Uint128::from(100u128),
                    "custom_receiver".to_string()
                )),
                gas_limit: None,
                reply_on: ReplyOn::Never
            })
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-execute-bond")
                    .add_attributes(vec![
                        attr("action", "bond"),
                        attr("amount", "100"),
                        attr("receiver", "custom_receiver")
                    ])
            )
    );
}

#[test]
fn test_execute_bond_wrong_denom() {
    let mut deps = mock_dependencies(&[]);
    DENOM
        .save(deps.as_mut().storage, &"dNTRN".to_string())
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "some_sender",
            &[Coin {
                denom: "wron_denom".to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Bond { receiver: None },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::PaymentError(PaymentError::MissingDenom("untrn".to_string()))
    );
}

#[test]
fn test_execute_bond_no_funds() {
    let mut deps = mock_dependencies(&[]);
    DENOM
        .save(deps.as_mut().storage, &"dNTRN".to_string())
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("some_sender", &[]),
        ExecuteMsg::Bond { receiver: None },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::PaymentError(PaymentError::NoFunds {})
    );
}

#[test]
fn test_execute_unbond() {
    let mut deps = mock_dependencies(&[]);
    let denom = "dNTRN";
    DENOM
        .save(deps.as_mut().storage, &denom.to_string())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    UNBOND_ID.save(deps.as_mut().storage, &0u64).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "some_sender",
            &[Coin {
                denom: denom.to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Unbond { receiver: None },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: "withdrawal_voucher".to_string(),
                        msg: to_json_binary(&WithdrawalVoucherExecuteMsg::Mint {
                            token_id: "1".to_string(),
                            owner: "some_sender".to_string(),
                            token_uri: None,
                            extension: Some(WithdrawalVoucherMetadata {
                                name: "dNTRN voucher".to_string(),
                                description: Some("Withdrawal voucher".to_string()),
                                release_at: 1571798419u64,
                                amount: Uint128::from(100u128),
                                recipient: "some_sender".to_string()
                            })
                        })
                        .unwrap(),
                        funds: vec![]
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                },
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::submit_burn_tokens(
                        "dNTRN".to_string(),
                        Uint128::from(100u128)
                    )),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                }
            ])
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-execute-unbond")
                    .add_attributes(vec![
                        attr("action", "unbond"),
                        attr("amount", "100"),
                        attr("receiver", "some_sender")
                    ])
            )
    );
}

#[test]
fn test_execute_unbond_custom_receiver() {
    let mut deps = mock_dependencies(&[]);
    let denom = "dNTRN";
    DENOM
        .save(deps.as_mut().storage, &denom.to_string())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    UNBOND_ID.save(deps.as_mut().storage, &0u64).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "some_sender",
            &[Coin {
                denom: denom.to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Unbond {
            receiver: Some("custom_receiver".to_string()),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: "withdrawal_voucher".to_string(),
                        msg: to_json_binary(&WithdrawalVoucherExecuteMsg::Mint {
                            token_id: "1".to_string(),
                            owner: "custom_receiver".to_string(),
                            token_uri: None,
                            extension: Some(WithdrawalVoucherMetadata {
                                name: "dNTRN voucher".to_string(),
                                description: Some("Withdrawal voucher".to_string()),
                                release_at: 1571798419u64,
                                amount: Uint128::from(100u128),
                                recipient: "custom_receiver".to_string()
                            })
                        })
                        .unwrap(),
                        funds: vec![]
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                },
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::submit_burn_tokens(
                        "dNTRN".to_string(),
                        Uint128::from(100u128)
                    )),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                }
            ])
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-execute-unbond")
                    .add_attributes(vec![
                        attr("action", "unbond"),
                        attr("amount", "100"),
                        attr("receiver", "custom_receiver")
                    ])
            )
    );
}

#[test]
fn test_execute_unbond_wrong_denom() {
    let mut deps = mock_dependencies(&[]);
    let denom = "dNTRN";
    DENOM
        .save(deps.as_mut().storage, &denom.to_string())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    UNBOND_ID.save(deps.as_mut().storage, &0u64).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "some_sender",
            &[Coin {
                denom: "wron_denom".to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Unbond { receiver: None },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::PaymentError(PaymentError::MissingDenom("dNTRN".to_string()))
    );
}

#[test]
fn test_execute_unbond_no_funds() {
    let mut deps = mock_dependencies(&[]);
    let denom = "dNTRN";
    DENOM
        .save(deps.as_mut().storage, &denom.to_string())
        .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    UNBOND_ID.save(deps.as_mut().storage, &0u64).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("some_sender", &[]),
        ExecuteMsg::Bond { receiver: None },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::PaymentError(PaymentError::NoFunds {})
    );
}

#[test]
fn test_execute_receive_nft_withdraw() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            to_json_binary(&NftInfoResponse::<WithdrawalVoucherExtension> {
                token_uri: None,
                extension: Some(WithdrawalVoucherMetadata {
                    name: "dNTRN voucher".to_string(),
                    description: Some("Withdrawal voucher".to_string()),
                    release_at: 0u64,
                    amount: Uint128::from(100u128),
                    recipient: "recipient".to_string(),
                }),
            })
            .unwrap()
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("withdrawal_voucher", &[]),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: "recipient".to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&ReceiveNftMsg::Withdraw { receiver: None }).unwrap(),
        }),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Bank(BankMsg::Send {
                    to_address: "recipient".to_string(),
                    amount: vec![Coin {
                        denom: BASE_DENOM.to_string(),
                        amount: Uint128::from(100u128)
                    }]
                }),
                gas_limit: None,
                reply_on: ReplyOn::Never
            })
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-execute-receive_nft")
                    .add_attributes(vec![
                        attr("action", "receive_nft"),
                        attr("amount", "100"),
                        attr("to_address", "recipient")
                    ])
            )
    )
}

#[test]
fn test_execute_receive_nft_withdraw_custom_receiver() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            to_json_binary(&NftInfoResponse::<WithdrawalVoucherExtension> {
                token_uri: None,
                extension: Some(WithdrawalVoucherMetadata {
                    name: "dNTRN voucher".to_string(),
                    description: Some("Withdrawal voucher".to_string()),
                    release_at: 0u64,
                    amount: Uint128::from(100u128),
                    recipient: "recipient".to_string(),
                }),
            })
            .unwrap()
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("withdrawal_voucher", &[]),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: "recipient".to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&ReceiveNftMsg::Withdraw {
                receiver: Some("custom_receiver".to_string()),
            })
            .unwrap(),
        }),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: 0,
                msg: CosmosMsg::Bank(BankMsg::Send {
                    to_address: "custom_receiver".to_string(),
                    amount: vec![Coin {
                        denom: BASE_DENOM.to_string(),
                        amount: Uint128::from(100u128)
                    }]
                }),
                gas_limit: None,
                reply_on: ReplyOn::Never
            })
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-execute-receive_nft")
                    .add_attributes(vec![
                        attr("action", "receive_nft"),
                        attr("amount", "100"),
                        attr("to_address", "custom_receiver")
                    ])
            )
    )
}

#[test]
fn test_execute_receive_nft_withdraw_no_funds_required() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "withdrawal_voucher",
            &[Coin {
                denom: "some_denom".to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: "recipient".to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&ReceiveNftMsg::Withdraw {
                receiver: Some("custom_receiver".to_string()),
            })
            .unwrap(),
        }),
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::PaymentError(PaymentError::NonPayable {})
    );
}

#[test]
fn test_execute_receive_nft_withdraw_not_authorized() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            to_json_binary(&NftInfoResponse::<WithdrawalVoucherExtension> {
                token_uri: None,
                extension: Some(WithdrawalVoucherMetadata {
                    name: "dNTRN voucher".to_string(),
                    description: Some("Withdrawal voucher".to_string()),
                    release_at: 0u64,
                    amount: Uint128::from(100u128),
                    recipient: "recipient".to_string(),
                }),
            })
            .unwrap()
        });
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                unbonding_period: 1000,
                withdrawal_voucher: Addr::unchecked("withdrawal_voucher"),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("somebody", &[]),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: "recipient".to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&ReceiveNftMsg::Withdraw {
                receiver: Some("custom_receiver".to_string()),
            })
            .unwrap(),
        }),
    )
    .unwrap_err();
    assert_eq!(res, crate::error::ContractError::Unauthorized {});
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: "new_owner".to_string(),
            expiry: Some(cw_ownable::Expiration::Never {}),
        }),
    )
    .unwrap();
    execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("new_owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership {}),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            crate::msg::QueryMsg::Ownership {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(cosmwasm_std::Addr::unchecked("new_owner".to_string())),
            pending_expiry: None,
            pending_owner: None
        }
    );
}

#[test]
fn test_reply_create_denom() {
    let mut deps = mock_dependencies(&[]);
    let dntrn = "dNTRN";
    let denom_metadata = DenomMetadata {
        name: "name".to_string(),
        description: "description".to_string(),
        denom_units: vec![],
        base: "base".to_string(),
        display: "display".to_string(),
        symbol: "symbol".to_string(),
        uri: "uri".to_string(),
        uri_hash: "uri_hash".to_string(),
    };
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&FullDenomResponse {
            denom: dntrn.to_string(),
        })
        .unwrap()
    });
    DENOM
        .save(deps.as_mut().storage, &dntrn.to_string())
        .unwrap();
    TOKEN_METADATA
        .save(deps.as_mut().storage, &denom_metadata)
        .unwrap();
    EXPONENT.save(deps.as_mut().storage, &6u32).unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: CREATE_DENOM_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: 0,
                msg: create_stargate_msg(
                    "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata",
                    neutron_sdk::proto_types::osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata {
                        sender: mock_env().contract.address.to_string(),
                        metadata: Some(Metadata {
                            denom_units: vec![
                                DenomUnit {
                                    denom: dntrn.to_string(),
                                    exponent: 0,
                                    aliases: vec![],
                                },
                                DenomUnit {
                                    denom: denom_metadata.display.to_string(),
                                    exponent: 6,
                                    aliases: vec![],
                                },
                            ],
                            base: dntrn.to_string(),
                            display: denom_metadata.display,
                            name: denom_metadata.name,
                            description: denom_metadata.description,
                            symbol: denom_metadata.symbol,
                            uri: denom_metadata.uri,
                            uri_hash: denom_metadata.uri_hash,
                        }),
                    },
                ),
                gas_limit: None,
                reply_on: ReplyOn::Never
            })
            .add_attribute("full_denom", "dNTRN")
    )
}

#[test]
fn test_reply_unknown_reply_id() {
    let mut deps = mock_dependencies(&[]);
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 123,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, crate::error::ContractError::UnknownReplyId { id: 123 })
}
