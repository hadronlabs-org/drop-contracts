use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, ConfigOptional, CONFIG, FAILED_TRANSFERS, REPLY_RECEIVERS, TF_DENOM_TO_NFT_ID,
    UNBOND_REPLY_ID,
};
use cosmwasm_std::ReplyOn;
use cosmwasm_std::{
    attr, from_json,
    testing::MOCK_CONTRACT_ADDR,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    to_json_binary, Addr, ChannelResponse, Coin, CosmosMsg, Decimal, Decimal256, Event, IbcChannel,
    IbcEndpoint, IbcOrder, OwnedDeps, Response, SubMsg, Timestamp, Uint128, WasmMsg,
};
use drop_helpers::answer::response;
use drop_helpers::testing::mock_dependencies;
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::RequestPacketTimeoutHeight,
};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
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
        mock_info("owner", &vec![]),
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
        mock_info("random_sender", &vec![]),
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
        mock_info("owner", &vec![]),
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
        mock_info("random_sender", &vec![]),
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
            &vec![
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
            &vec![Coin {
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
            &vec![Coin {
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
            &vec![Coin {
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
            "failed_receiver".to_string(),
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
        .load(&deps.storage, "failed_receiver".to_string())
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
        mock_info("sender", &vec![]),
        ExecuteMsg::Retry {
            receiver: "failed_receiver".to_string(),
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
                        attr("receiver", "failed_receiver"),
                        attr("amount", "1denom1"),
                        attr("receiver", "failed_receiver"),
                        attr("amount", "2denom2"),
                        attr("receiver", "failed_receiver"),
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
                        receiver: "failed_receiver".to_string(),
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
                        receiver: "failed_receiver".to_string(),
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
                        receiver: "failed_receiver".to_string(),
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
            .load(&deps.storage, "failed_receiver".to_string())
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
            "failed_receiver".to_string(),
            &vec![Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128),
            }],
        )
        .unwrap();
    for _ in 0..FAILED_TRANSFERS
        .load(&deps.storage, "failed_receiver".to_string())
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
        mock_info("sender", &vec![]),
        ExecuteMsg::Retry {
            receiver: "failed_receiver".to_string(),
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
                        attr("receiver", "failed_receiver"),
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
                    receiver: "failed_receiver".to_string(),
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
            .load(&deps.storage, "failed_receiver".to_string())
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
            "failed_receiver".to_string(),
            &vec![Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128),
            }],
        )
        .unwrap();
    for _ in 0..FAILED_TRANSFERS
        .load(&deps.storage, "failed_receiver".to_string())
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
        mock_info("sender", &vec![]),
        ExecuteMsg::Retry {
            receiver: "failed_receiver".to_string(),
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
                        attr("receiver", "failed_receiver"),
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
                    receiver: "failed_receiver".to_string(),
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
            .load(&deps.storage, "failed_receiver".to_string())
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
        mock_info("sender", &vec![]),
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
            &vec![
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
            &vec![Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(1u128),
            }],
        ),
        ExecuteMsg::Withdraw {
            receiver: "receiver".to_string(),
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
                        attr("receiver", "receiver"),
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
                    receiver: "receiver".to_string(),
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
