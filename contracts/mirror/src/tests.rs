use crate::contract::{execute, instantiate, query, reply, sudo};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, FailedReceiverResponse, FungibleTokenPacketData, InstantiateMsg, QueryMsg,
};
use crate::state::{
    Config, ConfigOptional, BOND_REPLY_ID, BOND_REPLY_RECEIVER, CONFIG, FAILED_TRANSFERS,
    IBC_TRANSFER_REPLY_ID, REPLY_TRANSFER_COIN, SUDO_SEQ_ID_TO_COIN,
};
use cosmwasm_std::{
    attr, from_json,
    testing::{message_info, mock_env},
    to_json_binary, Binary, ChannelResponse, Coin, CosmosMsg, Event, IbcChannel, IbcEndpoint,
    IbcOrder, MsgResponse, Reply, ReplyOn, Response, SubMsg, SubMsgResponse, SubMsgResult, Uint128,
    WasmMsg,
};
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use drop_helpers::testing::mock_dependencies;
use drop_helpers::testing::MOCK_CONTRACT_ADDR;
use neutron_sdk::bindings::msg::{IbcFee, NeutronMsg};
use neutron_sdk::query::min_ibc_fee::MinIbcFeeResponse;
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, TransferSudoMsg};
use neutron_std::types::neutron::interchaintxs::v1::MsgSubmitTxResponse;
use prost::Message;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    let res = instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        InstantiateMsg {
            owner: None,
            core_contract: api.addr_make("core_contract").to_string(),
            source_channel: "source_channel".to_string(),
            source_port: "source_port".to_string(),
            ibc_timeout: 0u64,
            prefix: "prefix".to_string(),
        },
    )
    .unwrap();
    CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-instantiate").add_attributes(vec![
                attr("action", "instantiate"),
                attr("owner", api.addr_make("owner")),
                attr("core_contract", api.addr_make("core_contract")),
                attr("source_port", "source_port"),
                attr("source_channel", "source_channel"),
                attr("ibc_timeout", "0"),
                attr("prefix", "prefix"),
            ])
        )
    )
}

#[test]
fn test_execute_bond_invalid_prefix() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0u64,
                prefix: "cosmos".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
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
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
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
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
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
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(
            &api.addr_make("owner"),
            &[Coin {
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
        BOND_REPLY_RECEIVER.load(&deps.storage).unwrap(),
        "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()
    );
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: BOND_REPLY_ID,
                payload: Binary::default(),
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
                Event::new("crates.io:drop-staking__drop-mirror-execute_bond").add_attributes(
                    vec![
                        attr("action", "execute_bond"),
                        attr("receiver", "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6"),
                        attr("ref", ""),
                        attr("coin", "123denom")
                    ]
                )
            )
    );
}

#[test]
fn test_execute_update_config_unauthrozied() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("owner").as_str()),
    )
    .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract_0").to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("random_sender"), &[]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some(api.addr_make("core_contract_1").to_string()),
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
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("owner").as_str()),
    )
    .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract_0").to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some(api.addr_make("core_contract_1").to_string()),
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
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("owner").as_str()),
    )
    .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract_0").to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    deps.querier.add_ibc_channel_response(
        Some("source_channel_1".to_string()),
        Some("source_port_1".to_string()),
        ChannelResponse::new(Some(IbcChannel::new(
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
        ))),
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some(api.addr_make("core_contract_1").to_string()),
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
            Event::new("crates.io:drop-staking__drop-mirror-execute_update_config").add_attributes(
                vec![
                    attr("action", "execute_update_config"),
                    attr("core_contract", api.addr_make("core_contract_1")),
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
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("owner").as_str()),
    )
    .unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract_0").to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    deps.querier.add_ibc_channel_response(
        Some("source_channel_1".to_string()),
        Some("source_port_1".to_string()),
        ChannelResponse::new(Some(IbcChannel::new(
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
        ))),
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("random_sender"), &[]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some(api.addr_make("core_contract_1").to_string()),
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
    let api = deps.api;
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("owner").as_str()),
    )
    .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: api.addr_make("new_owner").to_string(),
            expiry: Some(cw_ownable::Expiration::Never {}),
        }),
    )
    .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("new_owner"), &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership {}),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap()).unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(api.addr_make("new_owner")),
            pending_expiry: None,
            pending_owner: None
        }
    );
}

#[test]
fn test_execute_retry_take_1_from_3() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract").to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
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
                    denom: "denom1".to_string(),
                    amount: Uint128::from(1u128),
                },
                Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(1u128),
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
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-mirror-execute_retry").add_attributes(
                    vec![
                        attr("action", "execute_retry"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("amount", "1denom2"),
                    ]
                )
            )
            .add_submessages(vec![SubMsg {
                id: IBC_TRANSFER_REPLY_ID,
                payload: Binary::default(),
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port".to_string(),
                    source_channel: "source_channel".to_string(),
                    token: Coin {
                        denom: "denom2".to_string(),
                        amount: Uint128::from(1u128)
                    },
                    sender: api.addr_make(MOCK_CONTRACT_ADDR).to_string(),
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
    assert_eq!(
        FAILED_TRANSFERS
            .load(
                &deps.storage,
                "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
            )
            .unwrap(),
        [
            Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128)
            },
            Coin {
                denom: "denom1".to_string(),
                amount: Uint128::from(1u128)
            }
        ]
    );
    assert_eq!(
        REPLY_TRANSFER_COIN.load(&deps.storage).unwrap(),
        Coin {
            denom: "denom2".to_string(),
            amount: Uint128::from(1u128)
        }
    )
}

#[test]
fn test_execute_retry_take_one() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract").to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
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
        message_info(&api.addr_make("sender"), &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-staking__drop-mirror-execute_retry").add_attributes(
                    vec![
                        attr("action", "execute_retry"),
                        attr("receiver", "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc"),
                        attr("amount", "1denom1"),
                    ]
                )
            )
            .add_submessages(vec![SubMsg {
                id: IBC_TRANSFER_REPLY_ID,
                payload: Binary::default(),
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port".to_string(),
                    source_channel: "source_channel".to_string(),
                    token: Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(1u128)
                    },
                    sender: api.addr_make(MOCK_CONTRACT_ADDR).to_string(),
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
        REPLY_TRANSFER_COIN.load(&deps.storage).unwrap(),
        Coin {
            denom: "denom1".to_string(),
            amount: Uint128::from(1u128)
        }
    )
}

#[test]
fn test_execute_retry_take_0() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-execute_retry")
                .add_attributes(vec![attr("action", "execute_retry"),])
        )
    );
}

#[test]
fn test_execute_retry_take_empty() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
            },
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
            &vec![],
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-execute_retry")
                .add_attributes(vec![attr("action", "execute_retry"),])
        )
    );
}

#[test]
fn test_execute_retry_none() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix".to_string(),
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[]),
        ExecuteMsg::Retry {
            receiver: "prefix1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqckwusc".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-execute_retry")
                .add_attributes(vec![attr("action", "execute_retry"),])
        )
    );
}

#[test]
fn test_query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("owner").as_str()),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap()).unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(api.addr_make("owner")),
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
    let funds_in_failed_transfers = vec![
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
            &funds_in_failed_transfers,
        )
        .unwrap();
    let res: Vec<FailedReceiverResponse> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
    assert_eq!(
        res,
        vec![FailedReceiverResponse {
            receiver: "receiver".to_string(),
            failed_transfers: funds_in_failed_transfers
        }]
    );
}

#[test]
fn test_query_all_failed_empty() {
    let deps = mock_dependencies(&[]);
    let res: Vec<FailedReceiverResponse> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
    assert_eq!(res, vec![]);
}

#[test]
fn test_query_failed_receiver() {
    let mut deps = mock_dependencies(&[]);
    let funds_in_failed_transfers = vec![
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
            &funds_in_failed_transfers,
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
            failed_transfers: funds_in_failed_transfers
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
fn test_execute_reply_finalize_bond() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract_0").to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    BOND_REPLY_RECEIVER
        .save(
            deps.as_mut().storage,
            &api.addr_make("reply_receiver").to_string(),
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
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: BOND_REPLY_ID,
            payload: Binary::default(),
            gas_used: 1000,
            #[allow(deprecated)]
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("tf_mint").add_attribute("amount", "100dasset")],
                data: None,
                msg_responses: vec![],
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessage(SubMsg {
                id: 0,
                payload: Binary::default(),
                msg: CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: "source_port_0".to_string(),
                    source_channel: "source_channel_0".to_string(),
                    token: Coin {
                        denom: "dasset".to_string(),
                        amount: Uint128::from(100u128)
                    },
                    sender: api.addr_make("cosmos2contract").to_string(),
                    receiver: api.addr_make("reply_receiver").to_string(),
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
                reply_on: ReplyOn::Success
            })
            .add_event(
                Event::new("crates.io:drop-staking__drop-mirror-reply_finalize_bond")
                    .add_attributes(vec![
                        attr("action", "reply_finalize_bond"),
                        attr("amount", "100dasset"),
                        attr("to_address", api.addr_make("reply_receiver")),
                        attr("source_port", "source_port_0"),
                        attr("source_channel", "source_channel_0"),
                        attr("ibc-timeout", "0")
                    ])
            )
    );
    assert_eq!(
        REPLY_TRANSFER_COIN.load(&deps.storage).unwrap(),
        Coin {
            denom: "dasset".to_string(),
            amount: Uint128::from(100u128),
        }
    )
}

#[test]
fn test_execute_reply_finalize_bond_no_tokens_minted() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    BOND_REPLY_RECEIVER
        .save(deps.as_mut().storage, &"reply_receiver".to_string())
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: BOND_REPLY_ID,
            payload: Binary::default(),
            gas_used: 1000,
            #[allow(deprecated)]
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
                msg_responses: vec![],
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NoTokensMinted {});
}

#[test]
fn test_execute_reply_finalize_bond_no_tokens_minted_amount_found() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract_0".to_string(),
                source_channel: "source_channel_0".to_string(),
                source_port: "source_port_0".to_string(),
                ibc_timeout: 0u64,
                prefix: "neutron_0".to_string(),
            },
        )
        .unwrap();
    BOND_REPLY_RECEIVER
        .save(deps.as_mut().storage, &"reply_receiver".to_string())
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: BOND_REPLY_ID,
            payload: Binary::default(),
            gas_used: 1000,
            #[allow(deprecated)]
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("tf_mint")],
                data: None,
                msg_responses: vec![],
            }),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NoTokensMintedAmountFound {});
}

#[test]
fn test_reply_store_seq_id_invalid_type() {
    let mut deps = mock_dependencies(&[]);
    REPLY_TRANSFER_COIN
        .save(
            deps.as_mut().storage,
            &Coin {
                denom: "reply_transfer_coin".to_string(),
                amount: Uint128::from(1u128),
            },
        )
        .unwrap();
    let res = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: IBC_TRANSFER_REPLY_ID,
            payload: Binary::default(),
            gas_used: 1000,
            #[allow(deprecated)]
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![Event::new("wasm").add_attribute("token_id", "1_neutron..._123")],
                data: None,
                msg_responses: vec![MsgResponse {
                    type_url: "/neutron.interchainquery.v1.MsgIbcTransferResponse".to_string(),
                    value: Binary::from("wrong_data".as_bytes()),
                }],
            }),
        },
    )
    .unwrap_err();
    assert!(format!("{}", res).contains("failed to parse response"));
    assert!(format!("{}", res).contains("failed to decode Protobuf message"));
}

#[test]
fn test_reply_store_seq_id() {
    let mut deps = mock_dependencies(&[]);
    REPLY_TRANSFER_COIN
        .save(
            deps.as_mut().storage,
            &Coin {
                denom: "reply_transfer_coin".to_string(),
                amount: Uint128::from(1u128),
            },
        )
        .unwrap();
    let res: Response<NeutronMsg> = reply(
        deps.as_mut(),
        mock_env(),
        Reply {
            id: IBC_TRANSFER_REPLY_ID,
            payload: Binary::default(),
            gas_used: 1000,
            #[allow(deprecated)]
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
                msg_responses: vec![MsgResponse {
                    type_url: "/neutron.interchainquery.v1.MsgIbcTransferResponse".to_string(),
                    value: Binary::from(
                        MsgSubmitTxResponse {
                            sequence_id: 0u64,
                            channel: "channel".to_string(),
                        }
                        .encode_to_vec(),
                    ),
                }],
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-mirror-reply_store_seq_id").add_attributes(
                vec![
                    attr("action", "store_seq_id"),
                    attr(
                        "popped",
                        Coin {
                            denom: "reply_transfer_coin".to_string(),
                            amount: Uint128::from(1u128),
                        }
                        .to_string()
                    )
                ]
            )
        )
    );
}

#[test]
fn test_execute_sudo_response() {
    let mut deps = mock_dependencies(&[]);
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "receiver1".to_string(),
            &vec![Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(100u128),
            }],
        )
        .unwrap();
    FAILED_TRANSFERS
        .save(
            deps.as_mut().storage,
            "receiver2".to_string(),
            &vec![Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(100u128),
            }],
        )
        .unwrap();
    SUDO_SEQ_ID_TO_COIN
        .save(
            deps.as_mut().storage,
            0u64,
            &Coin {
                denom: "denom".to_string(),
                amount: Uint128::from(100u128),
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
            data: Binary::from("".as_bytes()),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-staking__drop-mirror-sudo_response"
        ))
    );
    let all_failed: Vec<FailedReceiverResponse> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
    assert_eq!(
        all_failed,
        vec![
            FailedReceiverResponse {
                receiver: "receiver1".to_string(),
                failed_transfers: vec![Coin {
                    denom: "denom".to_string(),
                    amount: Uint128::from(100u128),
                }]
            },
            FailedReceiverResponse {
                receiver: "receiver2".to_string(),
                failed_transfers: vec![Coin {
                    denom: "denom".to_string(),
                    amount: Uint128::from(100u128),
                }]
            },
        ]
    );
    SUDO_SEQ_ID_TO_COIN.load(&deps.storage, 0u64).unwrap_err();
}

#[test]
fn test_execute_sudo_timeout() {
    // same as sudo-error
    let mut deps = mock_dependencies(&[]);
    {
        SUDO_SEQ_ID_TO_COIN
            .save(
                deps.as_mut().storage,
                0u64,
                &Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(100u128),
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
                            denom: "wrong_denom".to_string(),
                            amount: "100".to_string(),
                            sender: "sender".to_string(),
                            receiver: "receiver1".to_string(),
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
                "crates.io:drop-staking__drop-mirror-sudo_timeout"
            ))
        );
        let all_failed: Vec<FailedReceiverResponse> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
        assert_eq!(
            all_failed,
            vec![FailedReceiverResponse {
                receiver: "receiver1".to_string(),
                failed_transfers: vec![Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(100u128),
                }]
            }]
        );
    }
    {
        SUDO_SEQ_ID_TO_COIN
            .save(
                deps.as_mut().storage,
                0u64,
                &Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(100u128),
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
                            denom: "wrong_denom".to_string(),
                            amount: "100".to_string(),
                            sender: "sender".to_string(),
                            receiver: "receiver1".to_string(),
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
                "crates.io:drop-staking__drop-mirror-sudo_timeout"
            ))
        );
        let all_failed: Vec<FailedReceiverResponse> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
        assert_eq!(
            all_failed,
            vec![FailedReceiverResponse {
                receiver: "receiver1".to_string(),
                failed_transfers: vec![
                    Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(100u128),
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: Uint128::from(100u128),
                    }
                ]
            }]
        );
    }
    {
        SUDO_SEQ_ID_TO_COIN
            .save(
                deps.as_mut().storage,
                0u64,
                &Coin {
                    denom: "denom2".to_string(),
                    amount: Uint128::from(200u128),
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
                            denom: "denom2".to_string(),
                            amount: "200".to_string(),
                            sender: "sender".to_string(),
                            receiver: "receiver1".to_string(),
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
                "crates.io:drop-staking__drop-mirror-sudo_timeout"
            ))
        );
        let all_failed: Vec<FailedReceiverResponse> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
        assert_eq!(
            all_failed,
            vec![FailedReceiverResponse {
                receiver: "receiver1".to_string(),
                failed_transfers: vec![
                    Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(100u128),
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: Uint128::from(100u128),
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: Uint128::from(200u128),
                    }
                ]
            }]
        );
    }
    {
        SUDO_SEQ_ID_TO_COIN
            .save(
                deps.as_mut().storage,
                0u64,
                &Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(300u128),
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
                            denom: "denom1".to_string(),
                            amount: "300".to_string(),
                            sender: "sender".to_string(),
                            receiver: "receiver2".to_string(),
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
                "crates.io:drop-staking__drop-mirror-sudo_timeout"
            ))
        );
        let all_failed: Vec<FailedReceiverResponse> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
        assert_eq!(
            all_failed,
            vec![
                FailedReceiverResponse {
                    receiver: "receiver1".to_string(),
                    failed_transfers: vec![
                        Coin {
                            denom: "denom1".to_string(),
                            amount: Uint128::from(100u128),
                        },
                        Coin {
                            denom: "denom2".to_string(),
                            amount: Uint128::from(100u128),
                        },
                        Coin {
                            denom: "denom2".to_string(),
                            amount: Uint128::from(200u128),
                        }
                    ]
                },
                FailedReceiverResponse {
                    receiver: "receiver2".to_string(),
                    failed_transfers: vec![Coin {
                        denom: "denom1".to_string(),
                        amount: Uint128::from(300u128),
                    },]
                },
            ]
        );
    }
    {
        SUDO_SEQ_ID_TO_COIN
            .save(
                deps.as_mut().storage,
                0u64,
                &Coin {
                    denom: "denom1".to_string(),
                    amount: Uint128::from(200u128),
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
                            denom: "wrong_denom".to_string(),
                            amount: "200".to_string(),
                            sender: "sender".to_string(),
                            receiver: "receiver2".to_string(),
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
                "crates.io:drop-staking__drop-mirror-sudo_timeout"
            ))
        );
        let all_failed: Vec<FailedReceiverResponse> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::AllFailed {}).unwrap()).unwrap();
        assert_eq!(
            all_failed,
            vec![
                FailedReceiverResponse {
                    receiver: "receiver1".to_string(),
                    failed_transfers: vec![
                        Coin {
                            denom: "denom1".to_string(),
                            amount: Uint128::from(100u128),
                        },
                        Coin {
                            denom: "denom2".to_string(),
                            amount: Uint128::from(100u128),
                        },
                        Coin {
                            denom: "denom2".to_string(),
                            amount: Uint128::from(200u128),
                        }
                    ]
                },
                FailedReceiverResponse {
                    receiver: "receiver2".to_string(),
                    failed_transfers: vec![
                        Coin {
                            denom: "denom1".to_string(),
                            amount: Uint128::from(300u128),
                        },
                        Coin {
                            denom: "denom1".to_string(),
                            amount: Uint128::from(200u128),
                        }
                    ]
                },
            ]
        );
    }
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
