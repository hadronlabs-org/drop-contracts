use crate::contract::{execute, instantiate, query, reply};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, FailedReceiverResponse, InstantiateMsg, QueryMsg, UnbondReadyListResponseItem,
};
use crate::state::{
    Config, ConfigOptional, CONFIG, FAILED_TRANSFERS, REPLY_RECEIVERS, TF_DENOM_TO_NFT_ID,
    UNBOND_REPLY_ID,
};
use cosmwasm_std::{
    attr, from_json,
    testing::MOCK_CONTRACT_ADDR,
    testing::{mock_env, mock_info},
    to_json_binary, ChannelResponse, Coin, CosmosMsg, Event, IbcChannel, IbcEndpoint, IbcOrder,
    Reply, ReplyOn, Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use drop_helpers::testing::mock_dependencies;
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::RequestPacketTimeoutHeight,
};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_ibc_channel_response(
        Some("source_channel".to_string()),
        Some("source_port".to_string()),
        ChannelResponse {
            channel: Some(IbcChannel::new(
                IbcEndpoint {
                    port_id: "source_port".to_string(),
                    channel_id: "source_channel".to_string(),
                },
                IbcEndpoint {
                    port_id: "source_port".to_string(),
                    channel_id: "source_channel".to_string(),
                },
                IbcOrder::Ordered,
                "version".to_string(),
                "connection_id".to_string(),
            )),
        },
    );
    let res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        InstantiateMsg {
            owner: None,
            core_contract: "core_contract".to_string(),
            withdrawal_manager: "withdrawal_manager".to_string(),
            withdrawal_voucher: "withdrawal_voucher".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 12345,
            ibc_denom: "ibc_denom".to_string(),
            prefix: "prefix".to_string(),
            retry_limit: 10,
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-unbonding-mirror-instantiate").add_attributes(
                vec![
                    attr("action", "instantiate"),
                    attr("owner", "owner"),
                    attr("core_contract", "core_contract"),
                    attr("withdrawal_manager", "withdrawal_manager"),
                    attr("withdrawal_voucher", "withdrawal_voucher"),
                    attr("source_port", "source_port"),
                    attr("source_channel", "source_channel"),
                    attr("ibc_timeout", "12345"),
                    attr("ibc_denom", "ibc_denom"),
                    attr("prefix", "prefix"),
                    attr("retry_limit", "10"),
                ]
            )
        )
    );
    assert_eq!(
        CONFIG.load(deps.as_ref().storage).unwrap(),
        Config {
            core_contract: "core_contract".to_string(),
            withdrawal_manager: "withdrawal_manager".to_string(),
            withdrawal_voucher: "withdrawal_voucher".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 12345,
            prefix: "prefix".to_string(),
            ibc_denom: "ibc_denom".to_string(),
            retry_limit: 10,
        }
    );
    assert_eq!(UNBOND_REPLY_ID.load(deps.as_ref().storage).unwrap(), 0u64);
}

#[test]
fn test_execute_update_config_source_channel_not_found() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager1".to_string(),
                withdrawal_voucher: "withdrawal_voucher1".to_string(),
                source_port: "source_port1".to_string(),
                source_channel: "source_channel1".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix1".to_string(),
                ibc_denom: "ibc_denom1".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract2".to_string()),
                withdrawal_manager: Some("withdrawal_manager2".to_string()),
                withdrawal_voucher: Some("withdrawal_voucher2".to_string()),
                source_port: Some("source_port2".to_string()),
                source_channel: Some("source_channel2".to_string()),
                ibc_timeout: Some(54321),
                prefix: Some("prefix2".to_string()),
                ibc_denom: Some("ibc_denom2".to_string()),
                retry_limit: Some(1),
            },
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::SourceChannelNotFound {});
}

#[test]
fn test_execute_update_config_unauthrozied() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("random_sender", &[]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract2".to_string()),
                withdrawal_manager: Some("withdrawal_manager2".to_string()),
                withdrawal_voucher: Some("withdrawal_voucher2".to_string()),
                source_port: Some("source_port2".to_string()),
                source_channel: Some("source_channel2".to_string()),
                ibc_timeout: Some(54321),
                prefix: Some("prefix2".to_string()),
                ibc_denom: Some("ibc_denom2".to_string()),
                retry_limit: Some(1),
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
fn test_execute_update_config() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager1".to_string(),
                withdrawal_voucher: "withdrawal_voucher1".to_string(),
                source_port: "source_port1".to_string(),
                source_channel: "source_channel1".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix1".to_string(),
                ibc_denom: "ibc_denom1".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    deps.querier.add_ibc_channel_response(
        Some("source_channel2".to_string()),
        Some("source_port2".to_string()),
        ChannelResponse {
            channel: Some(IbcChannel::new(
                IbcEndpoint {
                    port_id: "source_port2".to_string(),
                    channel_id: "source_channel2".to_string(),
                },
                IbcEndpoint {
                    port_id: "source_port2".to_string(),
                    channel_id: "source_channel2".to_string(),
                },
                IbcOrder::Ordered,
                "version".to_string(),
                "connection_id".to_string(),
            )),
        },
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract2".to_string()),
                withdrawal_manager: Some("withdrawal_manager2".to_string()),
                withdrawal_voucher: Some("withdrawal_voucher2".to_string()),
                source_port: Some("source_port2".to_string()),
                source_channel: Some("source_channel2".to_string()),
                ibc_timeout: Some(54321),
                prefix: Some("prefix2".to_string()),
                ibc_denom: Some("ibc_denom2".to_string()),
                retry_limit: Some(1),
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_update_config")
                .add_attributes(vec![
                    attr("action", "execute_update_config"),
                    attr("retry_limit", "1"),
                    attr("core_contract", "core_contract2"),
                    attr("withdrawal_manager", "withdrawal_manager2"),
                    attr("withdrawal_voucher", "withdrawal_voucher2"),
                    attr("ibc_timeout", "54321"),
                    attr("ibc_denom", "ibc_denom2"),
                    attr("prefix", "prefix2"),
                    attr("source_port", "source_port2"),
                    attr("source_channel", "source_channel2"),
                ])
        )
    );
    assert_eq!(
        CONFIG.load(&deps.storage).unwrap(),
        Config {
            core_contract: "core_contract2".to_string(),
            withdrawal_manager: "withdrawal_manager2".to_string(),
            withdrawal_voucher: "withdrawal_voucher2".to_string(),
            source_port: "source_port2".to_string(),
            source_channel: "source_channel2".to_string(),
            ibc_timeout: 54321,
            prefix: "prefix2".to_string(),
            ibc_denom: "ibc_denom2".to_string(),
            retry_limit: 1,
        }
    );
}

#[test]
fn test_execute_unbond_not_no_funds() {
    let mut deps = mock_dependencies(&[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("random_sender", &[]),
        ExecuteMsg::Unbond {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::PaymentError(cw_utils::PaymentError::NoFunds {})
    )
}

#[test]
fn test_execute_unbond_not_one_coin() {
    let mut deps = mock_dependencies(&[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "random_sender",
            &[
                Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(100u128),
                },
                Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(200u128),
                },
            ],
        ),
        ExecuteMsg::Unbond {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::PaymentError(cw_utils::PaymentError::MultipleDenoms {})
    )
}

#[test]
fn test_execute_unbond_invalid_prefix() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "random_sender",
            &[Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Unbond {
            receiver: "invalid_address".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::InvalidPrefix {})
}

#[test]
fn test_execute_unbond_wrong_receiver_address() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "random_sender",
            &[Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Unbond {
            receiver: "prefix1invalid_address".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::WrongReceiverAddress {})
}

#[test]
fn test_execute_unbond() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    UNBOND_REPLY_ID.save(deps.as_mut().storage, &0u64).unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "random_sender",
            &[Coin {
                denom: "dAsset".to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Unbond {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_unbond")
                    .add_attribute("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc")
            )
            .add_submessage(SubMsg {
                id: 1,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "core_contract".to_string(),
                    msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Unbond {})
                        .unwrap(),
                    funds: vec![Coin {
                        denom: "dAsset".to_string(),
                        amount: Uint128::from(100u128)
                    }]
                }),
                gas_limit: None,
                reply_on: cosmwasm_std::ReplyOn::Success
            })
    );
    assert_eq!(UNBOND_REPLY_ID.load(&deps.storage).unwrap(), 1u64);
    assert_eq!(
        REPLY_RECEIVERS.load(&deps.storage, 1u64).unwrap(),
        "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string()
    );
}

#[test]
fn test_execute_retry_invalid_prefix() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 3,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Retry {
            receiver: "invalid_prefix".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::InvalidPrefix {});
}

#[test]
fn test_execute_retry_wrong_receiver_address() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 3,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1invalid_address".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::WrongReceiverAddress {});
}

#[test]
fn test_execute_retry_take_less() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 3,
            },
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
            &vec![
                Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(1u128),
                },
                Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(2u128),
                },
                Coin {
                    denom: "denom3".to_string(),
                    amount: Uint128::from(3u128),
                },
                Coin {
                    denom: "denom4".to_string(),
                    amount: Uint128::from(4u128),
                },
            ],
        )
        .unwrap();
    for _ in 0..FAILED_TRANSFERS
        .load(
            &deps.storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        )
        .unwrap()
        .len()
    {
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
    }
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_retry")
                    .add_attributes(vec![
                        attr("action", "execute_retry"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("amount", "1denom1"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("amount", "2denom2"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("amount", "3denom3"),
                    ])
            )
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                        source_port: "source_port".to_string(),
                        source_channel: "source_channel".to_string(),
                        token: Coin {
                            denom: "denom1".to_string(),
                            amount: Uint128::from(1u128)
                        },
                        sender: MOCK_CONTRACT_ADDR.to_string(),
                        receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
                        timeout_height: RequestPacketTimeoutHeight {
                            revision_number: None,
                            revision_height: None,
                        },
                        timeout_timestamp: 1571809764879305533,
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
                    reply_on: ReplyOn::Never,
                },
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                        source_port: "source_port".to_string(),
                        source_channel: "source_channel".to_string(),
                        token: Coin {
                            denom: "denom2".to_string(),
                            amount: Uint128::from(2u128)
                        },
                        sender: MOCK_CONTRACT_ADDR.to_string(),
                        receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
                        timeout_height: RequestPacketTimeoutHeight {
                            revision_number: None,
                            revision_height: None,
                        },
                        timeout_timestamp: 1571809764879305533,
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
                    reply_on: ReplyOn::Never,
                },
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                        source_port: "source_port".to_string(),
                        source_channel: "source_channel".to_string(),
                        token: Coin {
                            denom: "denom3".to_string(),
                            amount: Uint128::from(3u128)
                        },
                        sender: MOCK_CONTRACT_ADDR.to_string(),
                        receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
                        timeout_height: RequestPacketTimeoutHeight {
                            revision_number: None,
                            revision_height: None,
                        },
                        timeout_timestamp: 1571809764879305533,
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
                    reply_on: ReplyOn::Never,
                }
            ])
    );
    assert_eq!(
        FAILED_TRANSFERS
            .load(
                &deps.storage,
                "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string()
            )
            .unwrap(),
        vec![Coin {
            denom: "denom4".to_string(),
            amount: Uint128::from(4u128)
        }]
    );
}

#[test]
fn test_execute_retry_take_bigger() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 3,
            },
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
            &vec![Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128),
            }],
        )
        .unwrap();
    for _ in 0..FAILED_TRANSFERS
        .load(
            &deps.storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        )
        .unwrap()
        .len()
    {
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
    }
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_retry")
                    .add_attributes(vec![
                        attr("action", "execute_retry"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("amount", "1denom1"),
                    ])
            )
            .add_submessages(vec![SubMsg {
                id: 0,
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port".to_string(),
                    source_channel: "source_channel".to_string(),
                    token: Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(1u128)
                    },
                    sender: MOCK_CONTRACT_ADDR.to_string(),
                    receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None,
                    },
                    timeout_timestamp: 1571809764879305533,
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
                reply_on: ReplyOn::Never,
            },])
    );
    assert_eq!(
        FAILED_TRANSFERS
            .load(
                &deps.storage,
                "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string()
            )
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn test_execute_retry_take_equal() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 1,
            },
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
            &vec![Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128),
            }],
        )
        .unwrap();
    for _ in 0..FAILED_TRANSFERS
        .load(
            &deps.storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        )
        .unwrap()
        .len()
    {
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
    }
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_retry")
                    .add_attributes(vec![
                        attr("action", "execute_retry"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("amount", "1denom1"),
                    ])
            )
            .add_submessages(vec![SubMsg {
                id: 0,
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port".to_string(),
                    source_channel: "source_channel".to_string(),
                    token: Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(1u128)
                    },
                    sender: MOCK_CONTRACT_ADDR.to_string(),
                    receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None,
                    },
                    timeout_timestamp: 1571809764879305533,
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
                reply_on: ReplyOn::Never,
            },])
    );
    assert_eq!(
        FAILED_TRANSFERS
            .load(
                &deps.storage,
                "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string()
            )
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn test_execute_withdraw_no_funds() {
    let mut deps = mock_dependencies(&[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        ExecuteMsg::Withdraw {
            receiver: "receiver".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::PaymentError(cw_utils::PaymentError::NoFunds {})
    );
}

#[test]
fn test_execute_withdraw_extra_funds() {
    let mut deps = mock_dependencies(&[]);
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[
                Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(1u128),
                },
                Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(1u128),
                },
            ],
        ),
        ExecuteMsg::Withdraw {
            receiver: "receiver".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::PaymentError(cw_utils::PaymentError::MultipleDenoms {})
    );
}

#[test]
fn test_execute_withdraw_invalid_prefix() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 1,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128),
            }],
        ),
        ExecuteMsg::Withdraw {
            receiver: "invalid_prefix".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::InvalidPrefix {});
}

#[test]
fn test_execute_withdraw_wrong_receiver_address() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 1,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "sender",
            &[Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128),
            }],
        ),
        ExecuteMsg::Withdraw {
            receiver: "prefix1invalid_address".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::WrongReceiverAddress {});
}

#[test]
fn test_execute_withdraw() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 1,
            },
        )
        .unwrap();
    TF_DENOM_TO_NFT_ID
        .save(
            deps.as_mut().storage,
            "denom".to_string(),
            &"1_neutron1_123".to_string(),
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&cw721::AllNftInfoResponse {
                    access: cw721::OwnerOfResponse {
                        owner: "owner".to_string(),
                        approvals: vec![],
                    },
                    info: cw721::NftInfoResponse::<
                        drop_staking_base::msg::withdrawal_voucher::Extension,
                    > {
                        token_uri: Some("token_uri".to_string()),
                        extension: Some(drop_staking_base::state::withdrawal_voucher::Metadata {
                            name: "name".to_string(),
                            description: None,
                            attributes: None,
                            batch_id: "batch_id".to_string(),
                            amount: Uint128::from(100u128),
                        }),
                    },
                })
                .unwrap(),
            )
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
        mock_info(
            "sender",
            &[Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(1u128),
            }],
        ),
        ExecuteMsg::Withdraw {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_withdraw")
                    .add_attributes(vec![
                        attr("action", "execute_withdraw"),
                        attr("voucher_amount", "1denom"),
                        attr("withdrawal_manager", "withdrawal_manager"),
                        attr("withdrawal_voucher", "withdrawal_voucher"),
                        attr("source_port", "source_port"),
                        attr("source_channel", "source_channel"),
                        attr("ibc_timeout", "12345"),
                        attr("nft_amount", "100ibc_denom"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                    ])
            )
            .add_submessages(vec![SubMsg {
                id: 0,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "withdrawal_voucher".to_string(),
                    msg: to_json_binary(
                        &drop_staking_base::msg::withdrawal_voucher::ExecuteMsg::SendNft {
                            contract: "withdrawal_manager".to_string(),
                            token_id: "1_neutron1_123".to_string(),
                            msg: to_json_binary(
                                &drop_staking_base::msg::withdrawal_manager::ReceiveNftMsg::Withdraw {
                                    receiver: None,
                                },
                            ).unwrap(),
                        },
                    ).unwrap(),
                    funds: vec![]
                }),
                gas_limit: None,
                reply_on: ReplyOn::Never,
            },
            SubMsg {
                id: 0,
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port".to_string(),
                    source_channel: "source_channel".to_string(),
                    token: Coin {
                        denom: "ibc_denom".to_string(),
                        amount: Uint128::from(100u128)
                    },
                    sender: MOCK_CONTRACT_ADDR.to_string(),
                    receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None,
                    },
                    timeout_timestamp: 1571809764879305533,
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
                reply_on: ReplyOn::Never,
            },
            SubMsg {
                id: 0,
                msg: CosmosMsg::Custom(NeutronMsg::BurnTokens {
                    denom: "denom".to_string(),
                    amount: Uint128::from(1u128),
                    burn_from_address: "".to_string()
                }),
                gas_limit: None,
                reply_on: ReplyOn::Never,
            }])
    )
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
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    let config = Config {
        core_contract: "core_contract".to_string(),
        withdrawal_manager: "withdrawal_manager".to_string(),
        withdrawal_voucher: "withdrawal_voucher".to_string(),
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        ibc_timeout: 12345,
        prefix: "prefix".to_string(),
        ibc_denom: "ibc_denom".to_string(),
        retry_limit: 10,
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let res: Config =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(res, config);
}

#[test]
fn test_query_wrong_failed_receiver() {
    let deps = mock_dependencies(&[]);
    let res: Option<FailedReceiverResponse> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::FailedReceiver {
                receiver: "wrong_failed_receiver".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, None)
}

#[test]
fn test_query_failed_receiver() {
    let mut deps = mock_dependencies(&[]);
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "failed_receiver".to_string(),
            &vec![Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(100u128),
            }],
        )
        .unwrap();

    let res: Option<FailedReceiverResponse> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::FailedReceiver {
                receiver: "failed_receiver".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        Some(FailedReceiverResponse {
            receiver: "failed_receiver".to_string(),
            amount: vec![Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(100u128)
            }]
        })
    )
}

#[test]
fn test_query_all_failed() {
    let mut deps = mock_dependencies(&[]);
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "failed_receiver1".to_string(),
            &vec![Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(100u128),
            }],
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "failed_receiver2".to_string(),
            &vec![
                Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(300u128),
                },
                Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(100u128),
                },
            ],
        )
        .unwrap();
    let res: Vec<(String, Vec<Coin>)> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
    assert_eq!(
        res,
        vec![
            (
                "failed_receiver1".to_string(),
                vec![Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(100u128)
                }]
            ),
            (
                "failed_receiver2".to_string(),
                vec![
                    Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(300u128)
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: Uint128::from(100u128)
                    }
                ]
            )
        ]
    );
}

#[test]
fn test_query_unbond_ready_true() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&cw721::AllNftInfoResponse {
                    access: cw721::OwnerOfResponse {
                        owner: "owner".to_string(),
                        approvals: vec![],
                    },
                    info: cw721::NftInfoResponse::<
                        drop_staking_base::msg::withdrawal_voucher::Extension,
                    > {
                        token_uri: Some("token_uri".to_string()),
                        extension: Some(drop_staking_base::state::withdrawal_voucher::Metadata {
                            name: "name".to_string(),
                            description: None,
                            attributes: None,
                            batch_id: "0".to_string(),
                            amount: Uint128::from(100u128),
                        }),
                    },
                })
                .unwrap(),
            )
        });
    deps.querier.add_wasm_query_response("core_contract", |_| {
        cosmwasm_std::ContractResult::Ok(
            to_json_binary(&drop_staking_base::state::core::UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(0u128),
                expected_native_asset_amount: Uint128::from(0u128),
                expected_release_time: 0,
                total_unbond_items: 0,
                status: drop_staking_base::state::core::UnbondBatchStatus::Withdrawn,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: drop_staking_base::state::core::UnbondBatchStatusTimestamps {
                    new: 0u64,
                    unbond_requested: None,
                    unbond_failed: None,
                    unbonding: None,
                    withdrawing: None,
                    withdrawn: None,
                    withdrawing_emergency: None,
                    withdrawn_emergency: None,
                },
            })
            .unwrap(),
        )
    });
    let res: bool = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UnbondReady {
                nft_id: "nft_id".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(res);
}

#[test]
fn test_query_unbond_ready_false() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&cw721::AllNftInfoResponse {
                    access: cw721::OwnerOfResponse {
                        owner: "owner".to_string(),
                        approvals: vec![],
                    },
                    info: cw721::NftInfoResponse::<
                        drop_staking_base::msg::withdrawal_voucher::Extension,
                    > {
                        token_uri: Some("token_uri".to_string()),
                        extension: Some(drop_staking_base::state::withdrawal_voucher::Metadata {
                            name: "name".to_string(),
                            description: None,
                            attributes: None,
                            batch_id: "0".to_string(),
                            amount: Uint128::from(100u128),
                        }),
                    },
                })
                .unwrap(),
            )
        });
    deps.querier.add_wasm_query_response("core_contract", |_| {
        cosmwasm_std::ContractResult::Ok(
            to_json_binary(&drop_staking_base::state::core::UnbondBatch {
                total_dasset_amount_to_withdraw: Uint128::from(0u128),
                expected_native_asset_amount: Uint128::from(0u128),
                expected_release_time: 0,
                total_unbond_items: 0,
                status: drop_staking_base::state::core::UnbondBatchStatus::Withdrawing,
                slashing_effect: None,
                unbonded_amount: None,
                withdrawn_amount: None,
                status_timestamps: drop_staking_base::state::core::UnbondBatchStatusTimestamps {
                    new: 0u64,
                    unbond_requested: None,
                    unbond_failed: None,
                    unbonding: None,
                    withdrawing: None,
                    withdrawn: None,
                    withdrawing_emergency: None,
                    withdrawn_emergency: None,
                },
            })
            .unwrap(),
        )
    });
    let res: bool = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UnbondReady {
                nft_id: "nft_id".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res);
}

#[test]
fn test_query_unbond_ready_list() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    deps.querier
        .add_wasm_query_response("withdrawal_voucher", |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&cw721::TokensResponse {
                    tokens: vec![
                        "token1".to_string(),
                        "token2".to_string(),
                        "token3".to_string(),
                        "token4".to_string(),
                        "token5".to_string(),
                    ],
                })
                .unwrap(),
            )
        });
    for i in 0..5 {
        deps.querier
            .add_wasm_query_response("withdrawal_voucher", move |_| {
                cosmwasm_std::ContractResult::Ok(
                    to_json_binary(&cw721::AllNftInfoResponse {
                        access: cw721::OwnerOfResponse {
                            owner: "owner".to_string(),
                            approvals: vec![],
                        },
                        info: cw721::NftInfoResponse::<
                            drop_staking_base::msg::withdrawal_voucher::Extension,
                        > {
                            token_uri: Some("token_uri".to_string()),
                            extension: Some(
                                drop_staking_base::state::withdrawal_voucher::Metadata {
                                    name: "name".to_string(),
                                    description: None,
                                    attributes: None,
                                    batch_id: (i % 2).to_string(),
                                    amount: Uint128::from(100u128),
                                },
                            ),
                        },
                    })
                    .unwrap(),
                )
            });
        deps.querier
            .add_wasm_query_response("core_contract", move |_| {
                cosmwasm_std::ContractResult::Ok(
                    to_json_binary(&drop_staking_base::state::core::UnbondBatch {
                        total_dasset_amount_to_withdraw: Uint128::from(0u128),
                        expected_native_asset_amount: Uint128::from(0u128),
                        expected_release_time: 0,
                        total_unbond_items: 0,
                        status: vec![
                            drop_staking_base::state::core::UnbondBatchStatus::Withdrawing,
                            drop_staking_base::state::core::UnbondBatchStatus::Withdrawn,
                        ][i % 2],
                        slashing_effect: None,
                        unbonded_amount: None,
                        withdrawn_amount: None,
                        status_timestamps:
                            drop_staking_base::state::core::UnbondBatchStatusTimestamps {
                                new: 0u64,
                                unbond_requested: None,
                                unbond_failed: None,
                                unbonding: None,
                                withdrawing: None,
                                withdrawn: None,
                                withdrawing_emergency: None,
                                withdrawn_emergency: None,
                            },
                    })
                    .unwrap(),
                )
            });
    }
    let res: Vec<UnbondReadyListResponseItem> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UnbondReadyList {
                receiver: "receiver".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        vec![
            UnbondReadyListResponseItem {
                nft_id: "token1".to_string(),
                status: false,
            },
            UnbondReadyListResponseItem {
                nft_id: "token2".to_string(),
                status: true,
            },
            UnbondReadyListResponseItem {
                nft_id: "token3".to_string(),
                status: false,
            },
            UnbondReadyListResponseItem {
                nft_id: "token4".to_string(),
                status: true,
            },
            UnbondReadyListResponseItem {
                nft_id: "token5".to_string(),
                status: false,
            }
        ]
    )
}

#[test]
fn test_reply_no_nft_minted() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    REPLY_RECEIVERS
        .save(deps.as_mut().storage, 1u64, &"receiver".to_string())
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
    assert_eq!(res, ContractError::NoNFTMinted {});
}

#[test]
fn test_reply_no_nft_minted_found() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    REPLY_RECEIVERS
        .save(deps.as_mut().storage, 1u64, &"receiver".to_string())
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: 1,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm")],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NoNFTMintedFound {});
}

#[test]
fn test_reply() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager".to_string(),
                withdrawal_voucher: "withdrawal_voucher".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
                ibc_denom: "ibc_denom".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    REPLY_RECEIVERS
        .save(deps.as_mut().storage, 1u64, &"receiver".to_string())
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
                events: vec![Event::new("wasm").add_attribute("token_id", "1_neutron..._123")],
                data: None,
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-reply-finalize_unbond")
                    .add_attributes(vec![
                        attr("action", "reply-finalize_bond"),
                        attr("reply_id", "1"),
                        attr("nft", "1_neutron..._123"),
                        attr("to_address", "receiver"),
                        attr("source_port", "source_port"),
                        attr("source_channel", "source_channel"),
                        attr("ibc_timeout", "12345"),
                        attr("tf_denom", "factory/cosmos2contract/nft_1_123"),
                    ])
            )
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::MintTokens {
                        denom: "nft_1_123".to_string(),
                        amount: Uint128::from(1u128),
                        mint_to_address: "cosmos2contract".to_string()
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
                            denom: "factory/cosmos2contract/nft_1_123".to_string(),
                            amount: Uint128::from(1u128)
                        },
                        sender: "cosmos2contract".to_string(),
                        receiver: "receiver".to_string(),
                        timeout_height: RequestPacketTimeoutHeight {
                            revision_height: None,
                            revision_number: None,
                        },
                        timeout_timestamp: 1571809764879305533,
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
    );
    REPLY_RECEIVERS.load(&deps.storage, 1u64).unwrap_err();
    assert_eq!(
        TF_DENOM_TO_NFT_ID
            .load(
                &deps.storage,
                "factory/cosmos2contract/nft_1_123".to_string(),
            )
            .unwrap(),
        "1_neutron..._123".to_string()
    )
}
