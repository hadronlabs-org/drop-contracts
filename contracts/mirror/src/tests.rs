use crate::contract::{execute, instantiate, query, reply, sudo};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, FailedReceiverResponse, FungibleTokenPacketData, InstantiateMsg, QueryMsg,
};
use crate::state::{Config, ConfigOptional, CONFIG, FAILED_TRANSFERS, REPLY_RECEIVER};
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, ChannelResponse, Coin, CosmosMsg, Event, IbcChannel, IbcEndpoint, IbcMsg,
    IbcOrder, Reply, ReplyOn, Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use drop_helpers::testing::mock_dependencies;
use neutron_sdk::bindings::msg::{IbcFee, NeutronMsg};
use neutron_sdk::query::min_ibc_fee::MinIbcFeeResponse;
use neutron_sdk::sudo::msg::RequestPacketTimeoutHeight;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&vec![]);
    let res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        InstantiateMsg {
            owner: None,
            core_contract: "core_contract".to_string(),
            source_channel: "source_channel".to_string(),
            source_port: "source_port".to_string(),
            ibc_timeout: 0,
            prefix: "prefix".to_string(),
        },
    )
    .unwrap();
    CONFIG.load(&deps.storage).unwrap();
    REPLY_RECEIVER.load(&deps.storage).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-instantiate")
                .add_attributes(vec![attr("action", "instantiate"), attr("owner", "owner")])
        )
    )
}

#[test]
fn test_execute_bond_invalid_prefix() {
    let mut deps = mock_dependencies(&vec![]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0,
                prefix: "cosmos".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        ExecuteMsg::Bond {
            receiver: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            r#ref: None,
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::InvalidPrefix {});
}

#[test]
fn test_execute_bond_payment_error() {
    let mut deps = mock_dependencies(&vec![]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0,
                prefix: "neutron".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        ExecuteMsg::Bond {
            receiver: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            r#ref: None,
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::PaymentError(PaymentError::NoFunds {}));
}

#[test]
fn test_execute_bond_wrong_receiver_address() {
    let mut deps = mock_dependencies(&vec![]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0,
                prefix: "neutron".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        ExecuteMsg::Bond {
            receiver: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqwrong_receiver_address".to_string(),
            r#ref: None,
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::WrongReceiverAddress {});
}

#[test]
fn test_execute_bond() {
    let mut deps = mock_dependencies(&vec![]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0,
                prefix: "neutron".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "owner",
            &vec![Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(123u128),
            }],
        ),
        ExecuteMsg::Bond {
            receiver: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            r#ref: None,
        },
    )
    .unwrap();
    assert_eq!(
        REPLY_RECEIVER.load(&deps.storage).unwrap(),
        "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()
    );
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: 1,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "core_contract".to_string(),
                    msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Bond {
                        receiver: None,
                        r#ref: None
                    })
                    .unwrap(),
                    funds: vec![Coin {
                        denom: "denom".to_string(),
                        amount: Uint128::from(123u128)
                    }]
                }),
                gas_limit: None,
                reply_on: ReplyOn::Success
            })
            .add_event(
                Event::new("crates.io:drop-staking__drop-mirror-bond").add_attributes(vec![
                    attr("action", "bond"),
                    attr("receiver", "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6"),
                    attr("ref", ""),
                    attr("coin", "123denom")
                ])
            )
    );
}

#[test]
fn test_execute_update_config_unauthrozied() {
    let mut deps = mock_dependencies(&vec![]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("random_sender", &vec![]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract_1".to_string()),
                source_channel: Some("source_channel_1".to_string()),
                source_port: Some("source_port_1".to_string()),
                ibc_timeout: Some(1),
                prefix: Some("neutron_1".to_string()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
    );
}

#[test]
fn test_execute_update_config_souce_channel_error() {
    let mut deps = mock_dependencies(&vec![]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract_1".to_string()),
                source_channel: Some("source_channel_1".to_string()),
                source_port: Some("source_port_1".to_string()),
                ibc_timeout: Some(1),
                prefix: Some("neutron_1".to_string()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::SourceChannelNotFound {});
}

#[test]
fn test_execute_update_config() {
    let mut deps = mock_dependencies(&vec![]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    deps.querier.add_ibc_channel_response(
        Some("source_channel_1".to_string()),
        Some("source_port_1".to_string()),
        ChannelResponse {
            channel: Some(IbcChannel::new(
                IbcEndpoint {
                    port_id: "port_id".to_string(),
                    channel_id: "channel_id".to_string(),
                },
                IbcEndpoint {
                    port_id: "port_id".to_string(),
                    channel_id: "channel_id".to_string(),
                },
                IbcOrder::Unordered,
                "version".to_string(),
                "connection_id".to_string(),
            )),
        },
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract_1".to_string()),
                source_channel: Some("source_channel_1".to_string()),
                source_port: Some("source_port_1".to_string()),
                ibc_timeout: Some(1),
                prefix: Some("neutron_1".to_string()),
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-execute-update_config").add_attributes(
                vec![
                    attr("action", "execute-update_config"),
                    attr("core_contract", "core_contract_1"),
                    attr("ibc_timeout", "1"),
                    attr("prefix", "neutron_1"),
                    attr("source_port", "source_port_1"),
                    attr("source_channel", "source_channel_1")
                ]
            )
        )
    );
}

#[test]
fn test_execute_update_config_unauthorized() {
    let mut deps = mock_dependencies(&vec![]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    deps.querier.add_ibc_channel_response(
        Some("source_channel_1".to_string()),
        Some("source_port_1".to_string()),
        ChannelResponse {
            channel: Some(IbcChannel::new(
                IbcEndpoint {
                    port_id: "port_id".to_string(),
                    channel_id: "channel_id".to_string(),
                },
                IbcEndpoint {
                    port_id: "port_id".to_string(),
                    channel_id: "channel_id".to_string(),
                },
                IbcOrder::Unordered,
                "version".to_string(),
                "connection_id".to_string(),
            )),
        },
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("random_sender", &vec![]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract_1".to_string()),
                source_channel: Some("source_channel_1".to_string()),
                source_port: Some("source_port_1".to_string()),
                ibc_timeout: Some(1),
                prefix: Some("neutron_1".to_string()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::OwnershipError(OwnershipError::NotOwner {})
    );
}

#[test]
fn test_execute_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: "new_owner".to_string(),
            expiry: Some(cw_ownable::Expiration::Never {}),
        }),
    )
    .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("new_owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership {}),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap()).unwrap();
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
fn test_execute_retry_removal_from_empty_failed_transfers() {
    let mut deps = mock_dependencies(&vec![]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 0u64,
                prefix: "prefix".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("permissionless_sender", &vec![]),
        ExecuteMsg::Retry {
            receiver: "receiver_that_doesn't_exist".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-retry")
                .add_attribute("action", "execute_retry")
        )
    )
}

#[test]
fn test_execute_retry() {
    let mut deps = mock_dependencies(&vec![]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 0u64,
                prefix: "prefix".to_string(),
            },
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "receiver".to_string(),
            &vec![
                Coin {
                    denom: "token_denom1".to_string(),
                    amount: Uint128::from(100u128),
                },
                Coin {
                    denom: "token_denom2".to_string(),
                    amount: Uint128::from(300u128),
                },
            ],
        )
        .unwrap();
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: cosmwasm_std::coins(100, "untrn"),
                timeout_fee: cosmwasm_std::coins(200, "untrn"),
            },
        })
        .unwrap()
    });
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: cosmwasm_std::coins(100, "untrn"),
                timeout_fee: cosmwasm_std::coins(200, "untrn"),
            },
        })
        .unwrap()
    });
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("permissionless_sender", &vec![]),
        ExecuteMsg::Retry {
            receiver: "receiver".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                        source_port: "source_port".to_string(),
                        source_channel: "source_channel".to_string(),
                        token: Coin {
                            denom: "token_denom1".to_string(),
                            amount: Uint128::from(100u128)
                        },
                        sender: "cosmos2contract".to_string(),
                        receiver: "receiver".to_string(),
                        timeout_height: RequestPacketTimeoutHeight {
                            revision_height: None,
                            revision_number: None
                        },
                        timeout_timestamp: mock_env().block.time.nanos(),
                        memo: "".to_string(),
                        fee: IbcFee {
                            recv_fee: vec![],
                            ack_fee: vec![Coin {
                                denom: "untrn".to_string(),
                                amount: Uint128::from(100u128)
                            }],
                            timeout_fee: vec![Coin {
                                denom: "untrn".to_string(),
                                amount: Uint128::from(200u128)
                            }]
                        }
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                },
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                        source_port: "source_port".to_string(),
                        source_channel: "source_channel".to_string(),
                        token: Coin {
                            denom: "token_denom2".to_string(),
                            amount: Uint128::from(300u128)
                        },
                        sender: "cosmos2contract".to_string(),
                        receiver: "receiver".to_string(),
                        timeout_height: RequestPacketTimeoutHeight {
                            revision_height: None,
                            revision_number: None
                        },
                        timeout_timestamp: mock_env().block.time.nanos(),
                        memo: "".to_string(),
                        fee: IbcFee {
                            recv_fee: vec![],
                            ack_fee: vec![Coin {
                                denom: "untrn".to_string(),
                                amount: Uint128::from(100u128)
                            }],
                            timeout_fee: vec![Coin {
                                denom: "untrn".to_string(),
                                amount: Uint128::from(200u128)
                            }]
                        }
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                }
            ])
            .add_event(
                Event::new("crates.io:drop-staking__drop-mirror-retry").add_attributes(vec![
                    attr("action", "execute_retry"),
                    attr("receiver", "receiver"),
                    attr("amount", "100token_denom1"),
                    attr("receiver", "receiver"),
                    attr("amount", "300token_denom2"),
                ])
            )
    )
}

#[test]
fn test_query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap()).unwrap();
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
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    let config = Config {
        core_contract: "core_contract".to_string(),
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        ibc_timeout: 0u64,
        prefix: "prefix".to_string(),
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res: Config =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(res, config);
}

#[test]
fn test_query_all_failed() {
    let mut deps = mock_dependencies(&[]);
    let funds_in_debt = vec![
        Coin {
            denom: "token_denom1".to_string(),
            amount: Uint128::from(100u128),
        },
        Coin {
            denom: "token_denom2".to_string(),
            amount: Uint128::from(300u128),
        },
    ];
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "receiver".to_string(),
            &funds_in_debt,
        )
        .unwrap();
    let res: Vec<(String, Vec<Coin>)> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
    assert_eq!(res, vec![("receiver".to_string(), funds_in_debt)]);
}

#[test]
fn test_query_all_failed_empty() {
    let deps = mock_dependencies(&[]);
    let res: Vec<(String, Vec<Coin>)> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
    assert_eq!(res, vec![]);
}

#[test]
fn test_query_failed_receiver() {
    let mut deps = mock_dependencies(&[]);
    let funds_in_debt = vec![
        Coin {
            denom: "token_denom1".to_string(),
            amount: Uint128::from(100u128),
        },
        Coin {
            denom: "token_denom2".to_string(),
            amount: Uint128::from(300u128),
        },
    ];
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "receiver".to_string(),
            &funds_in_debt,
        )
        .unwrap();
    let res: Option<FailedReceiverResponse> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::FailedReceiver {
                receiver: "receiver".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        Some(FailedReceiverResponse {
            receiver: "receiver".to_string(),
            amount: funds_in_debt
        })
    );
}

#[test]
fn test_query_failed_receiver_empty() {
    let deps = mock_dependencies(&[]);
    let res: Option<FailedReceiverResponse> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::FailedReceiver {
                receiver: "receiver".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, None);
}

#[test]
fn test_execute_reply() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    REPLY_RECEIVER
        .save(deps.as_mut().storage, &"reply_receiver".to_string())
        .unwrap();
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: cosmwasm_std::coins(100, "untrn"),
                timeout_fee: cosmwasm_std::coins(200, "untrn"),
            },
        })
        .unwrap()
    });
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("tf_mint").add_attribute("amount", "100dasset")],
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
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port_0".to_string(),
                    source_channel: "source_channel_0".to_string(),
                    token: Coin {
                        denom: "dasset".to_string(),
                        amount: Uint128::from(100u128)
                    },
                    sender: "cosmos2contract".to_string(),
                    receiver: "reply_receiver".to_string(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None
                    },
                    timeout_timestamp: mock_env().block.time.nanos(),
                    memo: "".to_string(),
                    fee: IbcFee {
                        recv_fee: vec![],
                        ack_fee: cosmwasm_std::coins(100, "untrn"),
                        timeout_fee: cosmwasm_std::coins(200, "untrn"),
                    }
                }),
                gas_limit: None,
                reply_on: ReplyOn::Never
            })
            .add_event(
                Event::new("crates.io:drop-staking__drop-mirror-reply-finalize_bond")
                    .add_attributes(vec![
                        attr("action", "reply-finalize_bond"),
                        attr("id", "1"),
                        attr("amount", "100dasset"),
                        attr("to_address", "reply_receiver"),
                        attr("source_port", "source_port_0"),
                        attr("source_channel", "source_channel_0"),
                        attr("ibc-timeout", "0")
                    ])
            )
    )
}

#[test]
fn test_execute_reply_no_tokens_minted() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    REPLY_RECEIVER
        .save(deps.as_mut().storage, &"reply_receiver".to_string())
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NoTokensMinted {});
}

#[test]
fn test_execute_reply_no_tokens_minted_amount_found() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    REPLY_RECEIVER
        .save(deps.as_mut().storage, &"reply_receiver".to_string())
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("tf_mint")],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NoTokensMintedAmountFound {});
}
