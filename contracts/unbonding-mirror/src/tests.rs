use std::collections::VecDeque;

use crate::contract::{execute, instantiate, query, reply, sudo};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, FailedReceiverResponse, FungibleTokenPacketData, InstantiateMsg, QueryMsg,
};
use crate::state::{
    Config, ConfigOptional, CONFIG, FAILED_TRANSFERS, IBC_TRANSFER_SUDO_REPLY_ID,
    REPLY_TRANSFER_COINS, SUDO_SEQ_ID_TO_COIN, TF_DENOM_TO_NFT_ID, UNBOND_REPLY_ID,
    UNBOND_REPLY_RECEIVER, WITHDRAW_REPLY_ID, WITHDRAW_REPLY_RECEIVER,
};
use cosmwasm_std::{
    attr, from_json,
    testing::MOCK_CONTRACT_ADDR,
    testing::{mock_env, mock_info},
    to_json_binary, ChannelResponse, Coin, CosmosMsg, Event, IbcChannel, IbcEndpoint, IbcOrder,
    Reply, ReplyOn, Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use drop_helpers::testing::mock_dependencies;
use neutron_sdk::bindings::msg::MsgIbcTransferResponse;
use neutron_sdk::sudo::msg::{RequestPacket, TransferSudoMsg};
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
            retry_limit: 10,
        }
    );
    assert_eq!(
        REPLY_TRANSFER_COINS.load(deps.as_ref().storage).unwrap(),
        VecDeque::new()
    );
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
                id: UNBOND_REPLY_ID,
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
    assert_eq!(
        UNBOND_REPLY_RECEIVER.load(&deps.storage).unwrap(),
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
    REPLY_TRANSFER_COINS
        .save(deps.as_mut().storage, &VecDeque::new())
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
                    id: IBC_TRANSFER_SUDO_REPLY_ID,
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
                    reply_on: ReplyOn::Success,
                },
                SubMsg {
                    id: IBC_TRANSFER_SUDO_REPLY_ID,
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
                    reply_on: ReplyOn::Success,
                },
                SubMsg {
                    id: IBC_TRANSFER_SUDO_REPLY_ID,
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
                    reply_on: ReplyOn::Success,
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
    assert_eq!(
        REPLY_TRANSFER_COINS.load(&deps.storage).unwrap(),
        VecDeque::from_iter(
            [
                Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(1u128)
                },
                Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(2u128)
                },
                Coin {
                    denom: "denom3".to_string(),
                    amount: Uint128::from(3u128)
                }
            ]
            .iter()
            .cloned()
        )
    )
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
    REPLY_TRANSFER_COINS
        .save(deps.as_mut().storage, &VecDeque::new())
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
                id: IBC_TRANSFER_SUDO_REPLY_ID,
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
                reply_on: ReplyOn::Success,
            },])
    );
    FAILED_TRANSFERS
        .load(
            &deps.storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        )
        .unwrap_err();
    assert_eq!(
        REPLY_TRANSFER_COINS.load(&deps.storage).unwrap(),
        VecDeque::from_iter(
            [Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128)
            },]
            .iter()
            .cloned()
        )
    )
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
    REPLY_TRANSFER_COINS
        .save(deps.as_mut().storage, &VecDeque::new())
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
                id: IBC_TRANSFER_SUDO_REPLY_ID,
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
                reply_on: ReplyOn::Success,
            },])
    );
    FAILED_TRANSFERS
        .load(
            &deps.storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        )
        .unwrap_err();
    assert_eq!(
        REPLY_TRANSFER_COINS.load(&deps.storage).unwrap(),
        VecDeque::from_iter(
            [Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128)
            },]
            .iter()
            .cloned()
        )
    )
}

#[test]
fn test_execute_retry_take_0() {
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
                retry_limit: 0,
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
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_retry")
                .add_attributes(vec![attr("action", "execute_retry"),])
        )
    );
    assert_eq!(
        FAILED_TRANSFERS
            .load(
                &deps.storage,
                "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string()
            )
            .unwrap(),
        vec![Coin {
            denom: "denom1".to_string(),
            amount: Uint128::from(1u128),
        }]
    );
}

#[test]
fn test_execute_retry_none() {
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
                retry_limit: 0,
            },
        )
        .unwrap();
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
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_retry")
                .add_attributes(vec![attr("action", "execute_retry"),])
        )
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
                        attr("withdraw_reply_receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("burn", "1denom"),
                    ])
            )
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::BurnTokens {
                        denom: "denom".to_string(),
                        amount: Uint128::from(1u128),
                        burn_from_address: "".to_string()
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Never,
                },
                SubMsg {
                    id: WITHDRAW_REPLY_ID,
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
                    reply_on: ReplyOn::Success,
            }])
        );
    TF_DENOM_TO_NFT_ID
        .load(&deps.storage, "denom".to_string())
        .unwrap_err();
    assert_eq!(
        WITHDRAW_REPLY_RECEIVER.load(&deps.storage).unwrap(),
        "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string()
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
            debt: vec![Coin {
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
    let res: Vec<FailedReceiverResponse> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
    assert_eq!(
        res,
        [
            FailedReceiverResponse {
                receiver: "failed_receiver1".to_string(),
                debt: vec![Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(100u128)
                }]
            },
            FailedReceiverResponse {
                receiver: "failed_receiver2".to_string(),
                debt: vec![
                    Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(300u128)
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: Uint128::from(100u128)
                    }
                ]
            }
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
                retry_limit: 10,
            },
        )
        .unwrap();
    TF_DENOM_TO_NFT_ID
        .save(
            deps.as_mut().storage,
            "id".to_string(),
            &"nft_id".to_string(),
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
                id: "id".to_string(),
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
                retry_limit: 10,
            },
        )
        .unwrap();
    TF_DENOM_TO_NFT_ID
        .save(
            deps.as_mut().storage,
            "id".to_string(),
            &"nft_id".to_string(),
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
                id: "id".to_string(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!res);
}

#[test]
fn test_reply_finalize_withdraw_no_transfer_amount() {
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
                retry_limit: 10,
            },
        )
        .unwrap();
    WITHDRAW_REPLY_RECEIVER
        .save(
            deps.as_mut().storage,
            &"withdraw_reply_receiver".to_string(),
        )
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: WITHDRAW_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NoTransferEvent {});
}

#[test]
fn test_reply_finalize_withdraw_no_transfer_amount_found() {
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
                retry_limit: 10,
            },
        )
        .unwrap();
    WITHDRAW_REPLY_RECEIVER
        .save(
            deps.as_mut().storage,
            &"withdraw_reply_receiver".to_string(),
        )
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: WITHDRAW_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("transfer")],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NoTransferAmountFound {});
}

#[test]
fn test_reply_finalize_withdraw() {
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
                retry_limit: 10,
            },
        )
        .unwrap();
    WITHDRAW_REPLY_RECEIVER
        .save(
            deps.as_mut().storage,
            &"withdraw_reply_receiver".to_string(),
        )
        .unwrap();
    REPLY_TRANSFER_COINS
        .save(deps.as_mut().storage, &VecDeque::new())
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
            id: WITHDRAW_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("transfer").add_attribute("amount", "100ibc_denom")],
                data: None,
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-reply_finalize_withdraw")
                    .add_attributes(vec![
                        attr("source_port", "source_port"),
                        attr("source_channel", "source_channel"),
                        attr("receiver", "withdraw_reply_receiver"),
                        attr("timeout", "1571809764879305533"),
                        attr("amount", "100ibc_denom"),
                    ])
            )
            .add_submessages(vec![SubMsg {
                id: IBC_TRANSFER_SUDO_REPLY_ID,
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port".to_string(),
                    source_channel: "source_channel".to_string(),
                    token: Coin {
                        denom: "ibc_denom".to_string(),
                        amount: Uint128::from(100u128)
                    },
                    sender: "cosmos2contract".to_string(),
                    receiver: "withdraw_reply_receiver".to_string(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None
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
                reply_on: ReplyOn::Success,
            }])
    );
    assert_eq!(
        REPLY_TRANSFER_COINS
            .load(&deps.storage)
            .unwrap()
            .pop_front()
            .unwrap(),
        Coin {
            denom: "ibc_denom".to_string(),
            amount: Uint128::from(100u128)
        }
    );
}

#[test]
fn test_reply_finalize_unbond_no_nft_minted() {
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
                retry_limit: 10,
            },
        )
        .unwrap();
    UNBOND_REPLY_RECEIVER
        .save(
            deps.as_mut().storage,
            &"withdraw_reply_receiver".to_string(),
        )
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: UNBOND_REPLY_ID,
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
fn test_reply_finalize_unbond_no_nft_minted_found() {
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
                retry_limit: 10,
            },
        )
        .unwrap();
    UNBOND_REPLY_RECEIVER
        .save(
            deps.as_mut().storage,
            &"withdraw_reply_receiver".to_string(),
        )
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: UNBOND_REPLY_ID,
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
fn test_reply_finalize_unbond() {
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
                retry_limit: 10,
            },
        )
        .unwrap();
    UNBOND_REPLY_RECEIVER
        .save(
            deps.as_mut().storage,
            &"withdraw_reply_receiver".to_string(),
        )
        .unwrap();
    REPLY_TRANSFER_COINS
        .save(deps.as_mut().storage, &VecDeque::new())
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
            id: UNBOND_REPLY_ID,
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
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-reply_finalize_unbond")
                    .add_attributes(vec![
                        attr("action", "reply_finalize_bond"),
                        attr("nft", "1_neutron..._123"),
                        attr("to_address", "withdraw_reply_receiver"),
                        attr("source_port", "source_port"),
                        attr("source_channel", "source_channel"),
                        attr("ibc_timeout", "12345"),
                        attr("tf_denom", "factory/cosmos2contract/nft_1_123"),
                    ])
            )
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::CreateDenom {
                        subdenom: "nft_1_123".to_string()
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                },
                SubMsg {
                    id: 0,
                    msg: CosmosMsg::Custom(NeutronMsg::MintTokens {
                        denom: "factory/cosmos2contract/nft_1_123".to_string(),
                        amount: Uint128::from(1u128),
                        mint_to_address: "cosmos2contract".to_string()
                    }),
                    gas_limit: None,
                    reply_on: ReplyOn::Never
                },
                SubMsg {
                    id: IBC_TRANSFER_SUDO_REPLY_ID,
                    msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                        source_port: "source_port".to_string(),
                        source_channel: "source_channel".to_string(),
                        token: Coin {
                            denom: "factory/cosmos2contract/nft_1_123".to_string(),
                            amount: Uint128::from(1u128)
                        },
                        sender: "cosmos2contract".to_string(),
                        receiver: "withdraw_reply_receiver".to_string(),
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
                    reply_on: ReplyOn::Success
                }
            ])
    );
    assert_eq!(
        TF_DENOM_TO_NFT_ID
            .load(
                &deps.storage,
                "factory/cosmos2contract/nft_1_123".to_string(),
            )
            .unwrap(),
        "1_neutron..._123".to_string()
    );
    assert_eq!(
        REPLY_TRANSFER_COINS
            .load(&deps.storage)
            .unwrap()
            .pop_front()
            .unwrap(),
        Coin {
            denom: "factory/cosmos2contract/nft_1_123".to_string(),
            amount: Uint128::from(1u128)
        }
    );
}

#[test]
fn test_reply_store_seq_id_invalid_type() {
    let mut deps = mock_dependencies(&[]);
    REPLY_TRANSFER_COINS
        .save(
            deps.as_mut().storage,
            &VecDeque::from_iter(
                [Coin {
                    denom: "reply_transfer_coin".to_string(),
                    amount: Uint128::from(1u128),
                }]
                .iter()
                .cloned(),
            ),
        )
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: IBC_TRANSFER_SUDO_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm").add_attribute("token_id", "1_neutron..._123")],
                data: Some(to_json_binary(&"wrong_data".to_string()).unwrap()),
            }),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "failed to parse response: InvalidType".to_string()
        })
    );
}

#[test]
fn test_reply_store_seq_id() {
    let mut deps = mock_dependencies(&[]);
    REPLY_TRANSFER_COINS
        .save(
            deps.as_mut().storage,
            &VecDeque::from_iter(
                [Coin {
                    denom: "reply_transfer_coin".to_string(),
                    amount: Uint128::from(1u128),
                }]
                .iter()
                .cloned(),
            ),
        )
        .unwrap();
    let res: Response<NeutronMsg> = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: IBC_TRANSFER_SUDO_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(
                    to_json_binary(&MsgIbcTransferResponse {
                        sequence_id: 0u64,
                        channel: "channel".to_string(),
                    })
                    .unwrap(),
                ),
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-unbonding-mirror-reply_store_seq_id")
                .add_attributes(vec![
                    attr("action", "store_seq_id"),
                    attr(
                        "popped",
                        Coin {
                            denom: "reply_transfer_coin".to_string(),
                            amount: Uint128::from(1u128),
                        }
                        .to_string()
                    )
                ])
        )
    );
    assert_eq!(
        REPLY_TRANSFER_COINS.load(&deps.storage).unwrap(),
        VecDeque::new()
    );
}

#[test]
fn test_reply_store_seq_id_take_from_queue() {
    let mut deps = mock_dependencies(&[]);
    REPLY_TRANSFER_COINS
        .save(
            deps.as_mut().storage,
            &VecDeque::from_iter(
                [
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
                ]
                .iter()
                .cloned(),
            ),
        )
        .unwrap();
    for coin in REPLY_TRANSFER_COINS.load(deps.as_ref().storage).unwrap() {
        let res: Response<NeutronMsg> = reply(
            deps.as_mut(),
            mock_env(),
            Reply {
                id: IBC_TRANSFER_SUDO_REPLY_ID,
                result: SubMsgResult::Ok(SubMsgResponse {
                    events: vec![],
                    data: Some(
                        to_json_binary(&MsgIbcTransferResponse {
                            sequence_id: 0u64,
                            channel: "channel".to_string(),
                        })
                        .unwrap(),
                    ),
                }),
            },
        )
        .unwrap();
        assert_eq!(
            res,
            Response::new().add_event(
                Event::new("crates.io:drop-staking__drop-unbonding-mirror-reply_store_seq_id")
                    .add_attributes(vec![
                        attr("action", "store_seq_id"),
                        attr("popped", coin.to_string())
                    ])
            )
        );
    }
    assert_eq!(
        REPLY_TRANSFER_COINS.load(&deps.storage).unwrap(),
        VecDeque::new()
    );
}

#[test]
fn test_sudo_response() {
    let mut deps = mock_dependencies(&[]);
    SUDO_SEQ_ID_TO_COIN
        .save(
            deps.as_mut().storage,
            0u64,
            &Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(1u128),
            },
        )
        .unwrap();
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        TransferSudoMsg::Response {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: None,
                source_channel: None,
                destination_port: None,
                destination_channel: None,
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            data: to_json_binary(&"".to_string()).unwrap(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-staking__drop-unbonding-mirror-sudo_response"
        ))
    );
    SUDO_SEQ_ID_TO_COIN.load(&deps.storage, 0u64).unwrap_err();
}

#[test]
fn test_sudo_error_timeout_update_existing_denom_amount() {
    let mut deps = mock_dependencies(&[]);
    SUDO_SEQ_ID_TO_COIN
        .save(
            deps.as_mut().storage,
            0,
            &Coin {
                denom: "correct_denom".to_string(),
                amount: Uint128::from(123u128),
            },
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "receiver".to_string(),
            &vec![Coin {
                denom: "correct_denom".to_string(),
                amount: Uint128::from(100u128),
            }],
        )
        .unwrap();
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        TransferSudoMsg::Timeout {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: None,
                source_channel: None,
                destination_port: None,
                destination_channel: None,
                data: Some(
                    to_json_binary(&FungibleTokenPacketData {
                        denom: "denom".to_string(),
                        amount: "123".to_string(),
                        sender: "sender".to_string(),
                        receiver: "receiver".to_string(),
                        memo: None,
                    })
                    .unwrap(),
                ),
                timeout_height: None,
                timeout_timestamp: None,
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-staking__drop-unbonding-mirror-sudo_timeout"
        ))
    );
    assert_eq!(
        FAILED_TRANSFERS
            .load(&deps.storage, "receiver".to_string())
            .unwrap(),
        vec![
            Coin {
                denom: "correct_denom".to_string(),
                amount: Uint128::from(100u128),
            },
            Coin {
                denom: "correct_denom".to_string(),
                amount: Uint128::from(123u128),
            }
        ]
    );
    SUDO_SEQ_ID_TO_COIN.load(&deps.storage, 0u64).unwrap_err();
}

#[test]
fn test_sudo_error_timeout_create_new_schedule() {
    let mut deps = mock_dependencies(&[]);
    SUDO_SEQ_ID_TO_COIN
        .save(
            deps.as_mut().storage,
            0,
            &Coin {
                denom: "correct_denom".to_string(),
                amount: Uint128::from(123u128),
            },
        )
        .unwrap();
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        TransferSudoMsg::Timeout {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: None,
                source_channel: None,
                destination_port: None,
                destination_channel: None,
                data: Some(
                    to_json_binary(&FungibleTokenPacketData {
                        denom: "denom".to_string(),
                        amount: "123".to_string(),
                        sender: "sender".to_string(),
                        receiver: "receiver".to_string(),
                        memo: None,
                    })
                    .unwrap(),
                ),
                timeout_height: None,
                timeout_timestamp: None,
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-staking__drop-unbonding-mirror-sudo_timeout"
        ))
    );
    assert_eq!(
        FAILED_TRANSFERS
            .load(&deps.storage, "receiver".to_string())
            .unwrap(),
        vec![Coin {
            denom: "correct_denom".to_string(),
            amount: Uint128::from(123u128),
        }]
    );
    SUDO_SEQ_ID_TO_COIN.load(&deps.storage, 0u64).unwrap_err();
}

#[test]
fn test_sudo_error_timeout_add_new_denom() {
    let mut deps = mock_dependencies(&[]);
    SUDO_SEQ_ID_TO_COIN
        .save(
            deps.as_mut().storage,
            0,
            &Coin {
                denom: "correct_denom".to_string(),
                amount: Uint128::from(123u128),
            },
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "receiver".to_string(),
            &vec![Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(100u128),
            }],
        )
        .unwrap();
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        TransferSudoMsg::Timeout {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: None,
                source_channel: None,
                destination_port: None,
                destination_channel: None,
                data: Some(
                    to_json_binary(&FungibleTokenPacketData {
                        denom: "wrong_denom".to_string(),
                        amount: "123".to_string(),
                        sender: "sender".to_string(),
                        receiver: "receiver".to_string(),
                        memo: None,
                    })
                    .unwrap(),
                ),
                timeout_height: None,
                timeout_timestamp: None,
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-staking__drop-unbonding-mirror-sudo_timeout"
        ))
    );
    assert_eq!(
        FAILED_TRANSFERS
            .load(&deps.storage, "receiver".to_string())
            .unwrap(),
        vec![
            Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(100u128),
            },
            Coin {
                denom: "correct_denom".to_string(),
                amount: Uint128::from(123u128),
            }
        ]
    );
    SUDO_SEQ_ID_TO_COIN.load(&deps.storage, 0u64).unwrap_err();
}

#[test]
fn test_migrate_wrong_contract() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    cw2::set_contract_version(deps_mut.storage, "wrong_contract_name", "0.0.1").unwrap();

    let res =
        crate::contract::migrate(deps.as_mut(), mock_env(), crate::msg::MigrateMsg {}).unwrap_err();
    assert_eq!(
        res,
        ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}
