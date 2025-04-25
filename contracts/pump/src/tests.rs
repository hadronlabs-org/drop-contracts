use crate::{
    contract::{execute, instantiate, query, sudo},
    error::ContractError,
};
use cosmwasm_std::{
    coins, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Event, Response, SubMsg,
};
use drop_helpers::ica::IcaState;
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::pump::{Config, CONFIG, ICA};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        types::{Height, ProtobufAny},
    },
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, SudoMsg},
};
use prost::Message;

fn get_default_config() -> Config {
    Config {
        dest_address: Some(Addr::unchecked("dest_address")),
        dest_channel: Some("dest_channel".to_string()),
        dest_port: Some("dest_port".to_string()),
        connection_id: "connection".to_string(),
        refundee: Some(Addr::unchecked("refundee")),
        timeout: drop_staking_base::state::pump::PumpTimeout {
            local: Some(10u64),
            remote: 10u64,
        },
        local_denom: "local_denom".to_string(),
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = drop_staking_base::msg::pump::InstantiateMsg {
        dest_address: Some("dest_address".to_string()),
        dest_channel: Some("dest_channel".to_string()),
        dest_port: Some("dest_port".to_string()),
        connection_id: "connection".to_string(),
        refundee: Some("refundee".to_string()),
        timeout: drop_staking_base::state::pump::PumpTimeout {
            local: Some(10u64),
            remote: 10u64,
        },
        local_denom: "local_denom".to_string(),
        owner: Some("owner".to_string()),
    };
    let res = instantiate(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-neutron-contracts__drop-pump-instantiate"
        ).add_attributes(vec![
            ("contract_name", "crates.io:drop-neutron-contracts__drop-pump"),
            ("contract_version", "1.3.0"),
            ("msg", "InstantiateMsg { dest_address: Some(\"dest_address\"), dest_channel: Some(\"dest_channel\"), dest_port: Some(\"dest_port\"), connection_id: \"connection\", refundee: Some(\"refundee\"), timeout: PumpTimeout { local: Some(10), remote: 10 }, local_denom: \"local_denom\", owner: Some(\"owner\") }"),
            ("sender", "owner")
        ]))
    );
    assert_eq!(
        "owner",
        cw_ownable::get_ownership(deps.as_ref().storage)
            .unwrap()
            .owner
            .unwrap()
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, get_default_config());
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let msg = drop_staking_base::msg::pump::InstantiateMsg {
        dest_address: Some("dest_address".to_string()),
        dest_channel: Some("dest_channel".to_string()),
        dest_port: Some("dest_port".to_string()),
        connection_id: "connection".to_string(),
        refundee: Some("refundee".to_string()),
        timeout: drop_staking_base::state::pump::PumpTimeout {
            local: Some(0u64),
            remote: 0u64,
        },
        local_denom: "local_denom".to_string(),
        owner: Some("owner".to_string()),
    };
    let _ = instantiate(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::UpdateConfig {
            new_config: Box::new(drop_staking_base::msg::pump::UpdateConfigMsg {
                dest_address: Some("new_dest_address".to_string()),
                dest_channel: Some("new_dest_channel".to_string()),
                dest_port: Some("new_dest_port".to_string()),
                connection_id: Some("new_connection".to_string()),
                refundee: Some("new_refundee".to_string()),
                timeout: Some(drop_staking_base::state::pump::PumpTimeout {
                    local: Some(1u64),
                    remote: 1u64,
                }),
                local_denom: Some("new_local_denom".to_string()),
            }),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
    )
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = drop_staking_base::msg::pump::InstantiateMsg {
        dest_address: Some("dest_address".to_string()),
        dest_channel: Some("dest_channel".to_string()),
        dest_port: Some("dest_port".to_string()),
        connection_id: "connection_id".to_string(),
        refundee: Some("refundee".to_string()),
        timeout: drop_staking_base::state::pump::PumpTimeout {
            local: Some(0u64),
            remote: 0u64,
        },
        local_denom: "local_denom".to_string(),
        owner: Some("owner".to_string()),
    };
    let _ = instantiate(deps.as_mut(), mock_env(), mock_info("admin", &[]), msg).unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("admin")).unwrap();
    let msg = drop_staking_base::msg::pump::UpdateConfigMsg {
        dest_address: Some("new_dest_address".to_string()),
        dest_channel: Some("new_dest_channel".to_string()),
        dest_port: Some("new_dest_port".to_string()),
        connection_id: Some("new_connection_id".to_string()),
        refundee: Some("new_refundee".to_string()),
        timeout: Some(drop_staking_base::state::pump::PumpTimeout {
            local: Some(1u64),
            remote: 1u64,
        }),
        local_denom: Some("new_local_denom".to_string()),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::UpdateConfig {
            new_config: Box::new(msg.clone()),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-neutron-contracts__drop-pump-update_config").add_attributes(
                vec![
                    cosmwasm_std::attr("action", "update_config"),
                    cosmwasm_std::attr("dest_address", "new_dest_address"),
                    cosmwasm_std::attr("dest_channel", "new_dest_channel"),
                    cosmwasm_std::attr("dest_port", "new_dest_port"),
                    cosmwasm_std::attr("connection_id", "new_connection_id"),
                    cosmwasm_std::attr("refundee", "new_refundee"),
                    cosmwasm_std::attr("timeout", format!("{:?}", msg.timeout.unwrap())),
                    cosmwasm_std::attr("local_denom", "new_local_denom"),
                ]
            )
        )
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            dest_address: Some(Addr::unchecked("new_dest_address")),
            dest_channel: Some("new_dest_channel".to_string()),
            dest_port: Some("new_dest_port".to_string()),
            connection_id: "new_connection_id".to_string(),
            refundee: Some(Addr::unchecked("new_refundee")),
            timeout: drop_staking_base::state::pump::PumpTimeout {
                local: Some(1u64),
                remote: 1u64,
            },
            local_denom: "new_local_denom".to_string(),
        }
    );
}

#[test]
fn test_register_ica_no_fee() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let msg = drop_staking_base::msg::pump::ExecuteMsg::RegisterICA {};

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[]),
        msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidFunds {
            reason: "missing fee in denom local_denom".to_string()
        }
    );
    assert_eq!(ICA.load(deps.as_ref().storage).unwrap(), IcaState::None);
}

#[test]
fn test_register_ica() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let msg = drop_staking_base::msg::pump::ExecuteMsg::RegisterICA {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &coins(1000, "local_denom")),
        msg.clone(),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-register-ica")
                    .add_attributes(vec![
                        ("action", "register_ica"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_PUMP")
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Custom(
                NeutronMsg::register_interchain_account(
                    "connection".to_string(),
                    "drop_PUMP".to_string(),
                    Some(coins(1000, "local_denom")),
                )
            )))
    );
    // already asked for registration
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &coins(1000, "local_denom")),
        msg.clone(),
    );
    assert_eq!(
        res,
        Err(crate::error::ContractError::Std(
            cosmwasm_std::StdError::generic_err("ICA registration is in progress right now")
        ))
    );
    // reopen timeouted ICA
    ICA.set_timeout(deps.as_mut().storage).unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &coins(1000, "local_denom")),
        msg.clone(),
    )
    .unwrap();
    assert_eq!(
        ICA.load(deps.as_ref().storage).unwrap(),
        IcaState::InProgress
    );
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-register-ica")
                    .add_attributes(vec![
                        ("action", "register_ica"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_PUMP")
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Custom(
                NeutronMsg::register_interchain_account(
                    "connection".to_string(),
                    "drop_PUMP".to_string(),
                    Some(coins(1000, "local_denom")),
                )
            )))
    );
}

#[test]
fn test_execute_refund_no_refundee() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    let mut config = get_default_config();
    config.refundee = None;
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let err = execute(deps.as_mut(), mock_env(), mock_info("nobody", &[]), msg).unwrap_err();
    assert_eq!(err, ContractError::RefundeeIsNotSet {});
}

#[test]
fn test_execute_refund_success_refundee() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let res = execute(deps.as_mut(), mock_env(), mock_info("refundee", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-refund")
                    .add_attributes(vec![("action", "refund"), ("refundee", "refundee")])
            )
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "refundee".to_string(),
                amount: vec![Coin::new(200, "untrn")]
            }))
    );
}

#[test]
fn test_execute_refund() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-refund")
                    .add_attributes(vec![("action", "refund"), ("refundee", "refundee")])
            )
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "refundee".to_string(),
                amount: vec![Coin::new(200, "untrn")]
            }))
    );
}

#[test]
fn test_execute_refund_unauthorized() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        msg,
    )
    .unwrap_err();
    assert_eq!(err, crate::error::ContractError::Unauthorized {});
}

#[test]
fn test_push_no_destination_port() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: coins(100, "local_denom"),
                timeout_fee: coins(200, "local_denom"),
            },
        })
        .unwrap()
    });
    let mut config = get_default_config();
    config.dest_port = None;
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    ICA.set_address(deps.as_mut().storage, "some", "port", "channel")
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("somebody", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::Push {
            coins: vec![Coin::new(100u128, "remote_denom")],
        }
        .clone(),
    )
    .unwrap_err();
    assert_eq!(res, crate::error::ContractError::NoDestinationPort {});
}

#[test]
fn test_push_no_destintation_channel() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: coins(100, "local_denom"),
                timeout_fee: coins(200, "local_denom"),
            },
        })
        .unwrap()
    });
    let mut config = get_default_config();
    config.dest_channel = None;
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    ICA.set_address(deps.as_mut().storage, "some", "port", "channel")
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("somebody", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::Push {
            coins: vec![Coin::new(100u128, "remote_denom")],
        }
        .clone(),
    )
    .unwrap_err();
    assert_eq!(res, crate::error::ContractError::NoDestinationChannel {});
}

#[test]
fn test_push_no_destintation_address() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: coins(100, "local_denom"),
                timeout_fee: coins(200, "local_denom"),
            },
        })
        .unwrap()
    });
    let mut config = get_default_config();
    config.dest_address = None;
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    ICA.set_address(deps.as_mut().storage, "some", "port", "channel")
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("somebody", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::Push {
            coins: vec![Coin::new(100u128, "remote_denom")],
        }
        .clone(),
    )
    .unwrap_err();
    assert_eq!(res, crate::error::ContractError::NoDestinationAddress {});
}

#[test]
fn test_push() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: coins(100, "local_denom"),
                timeout_fee: coins(200, "local_denom"),
            },
        })
        .unwrap()
    });
    ICA.set_address(deps.as_mut().storage, "some", "port", "channel")
        .unwrap();
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("somebody", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::Push {
            coins: vec![Coin::new(100u128, "remote_denom")],
        }
        .clone(),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-push").add_attributes(
                    vec![
                        ("action", "push"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_PUMP"),
                        ("coins", "[Coin { 100 \"remote_denom\" }]"),
                    ]
                )
            )
            .add_message(CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection".to_string(),
                "drop_PUMP".to_string(),
                vec![ProtobufAny {
                    type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
                    value: Binary::from(
                        cosmos_sdk_proto::ibc::applications::transfer::v1::MsgTransfer {
                            source_port: "dest_port".to_string(),
                            source_channel: "dest_channel".to_string(),
                            token: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                denom: "remote_denom".to_string(),
                                amount: "100".to_string(),
                            }),
                            sender: "some".to_string(),
                            receiver: "dest_address".to_string(),
                            timeout_height: None,
                            timeout_timestamp: env.block.time.plus_seconds(10).nanos(),
                        }
                        .encode_to_vec(),
                    ),
                }],
                "".to_string(),
                10u64,
                IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(100, "local_denom"),
                    timeout_fee: coins(200, "local_denom")
                }
            )))
    );
}

#[test]
fn test_sudo_response_sequence_not_found() {
    let mut deps = mock_dependencies(&[]);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::Response {
            request: RequestPacket {
                sequence: None,
                source_port: Some("transfer".to_string()),
                source_channel: Some("channel-0".to_string()),
                destination_port: Some("transfer".to_string()),
                destination_channel: Some("channel-1".to_string()),
                timeout_height: Some(RequestPacketTimeoutHeight {
                    revision_height: Some(0u64),
                    revision_number: Some(0u64),
                }),
                data: Some(Binary::from([0; 0])),
                timeout_timestamp: Some(0u64),
            },
            data: Binary::from([0; 0]),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "sequence not found".to_string()
        })
    );
}

#[test]
#[allow(deprecated)]
fn test_sudo_response_ack_not_implemented() {
    let mut deps = mock_dependencies(&[]);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::Response {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: Some("transfer".to_string()),
                source_channel: Some("channel-0".to_string()),
                destination_port: Some("transfer".to_string()),
                destination_channel: Some("channel-1".to_string()),
                timeout_height: Some(RequestPacketTimeoutHeight {
                    revision_height: Some(0u64),
                    revision_number: Some(0u64),
                }),
                data: Some(Binary::from([0; 0])),
                timeout_timestamp: Some(0u64),
            },
            data: Binary::from(
                cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData {
                    data: vec![cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "something".to_string(),
                        data: vec![],
                    }],
                    msg_responses: vec![],
                }
                .encode_to_vec(),
            ),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(cosmwasm_std::StdError::generic_err(
            "This type of acknowledgement is not implemented"
        ))
    );
}

#[test]
#[allow(deprecated)]
fn test_sudo_response() {
    let mut deps = mock_dependencies(&[]);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::Response {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: Some("transfer".to_string()),
                source_channel: Some("channel-0".to_string()),
                destination_port: Some("transfer".to_string()),
                destination_channel: Some("channel-1".to_string()),
                timeout_height: Some(RequestPacketTimeoutHeight {
                    revision_height: Some(0u64),
                    revision_number: Some(0u64),
                }),
                data: Some(Binary::from([0; 0])),
                timeout_timestamp: Some(0u64),
            },
            data: Binary::from(
                cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData {
                    data: vec![cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/ibc.applications.transfer.v1.MsgTransferResponse".to_string(),
                        data: cosmos_sdk_proto::ibc::applications::transfer::v1::MsgTransferResponse {}.encode_to_vec(),
                    }],
                    msg_responses: vec![],
                }
                .encode_to_vec(),
            ),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-neutron-contracts__drop-pump-sudo-response".to_string()
            )
            .add_attributes(vec![
                cosmwasm_std::attr("action".to_string(), "sudo_response".to_string()),
                cosmwasm_std::attr("request_id".to_string(), "0".to_string())
            ])
        )
    );
}

#[test]
fn test_sudo_error_sequence_not_found() {
    let mut deps = mock_dependencies(&[]);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::Error {
            request: RequestPacket {
                sequence: None,
                source_port: Some("transfer".to_string()),
                source_channel: Some("channel-0".to_string()),
                destination_port: Some("transfer".to_string()),
                destination_channel: Some("channel-1".to_string()),
                timeout_height: Some(RequestPacketTimeoutHeight {
                    revision_height: Some(0u64),
                    revision_number: Some(0u64),
                }),
                data: Some(Binary::from([0; 0])),
                timeout_timestamp: Some(0u64),
            },
            details: "".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "sequence not found".to_string()
        })
    )
}

#[test]
fn test_sudo_error() {
    let mut deps = mock_dependencies(&[]);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::Error {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: Some("transfer".to_string()),
                source_channel: Some("channel-0".to_string()),
                destination_port: Some("transfer".to_string()),
                destination_channel: Some("channel-1".to_string()),
                timeout_height: Some(RequestPacketTimeoutHeight {
                    revision_height: Some(0u64),
                    revision_number: Some(0u64),
                }),
                data: Some(Binary::from([0; 0])),
                timeout_timestamp: Some(0u64),
            },
            details: "".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-neutron-contracts__drop-pump-sudo-error".to_string()
            )
            .add_attributes(vec![
                cosmwasm_std::attr("action".to_string(), "sudo_error".to_string()),
                cosmwasm_std::attr("request_id".to_string(), "0".to_string()),
                cosmwasm_std::attr("details".to_string(), "".to_string())
            ])
        )
    )
}

#[test]
fn test_sudo_timeout() {
    let mut deps = mock_dependencies(&[]);
    assert_eq!(ICA.load(deps.as_mut().storage).unwrap(), IcaState::None);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::Timeout {
            request: RequestPacket {
                sequence: Some(0u64),
                source_port: Some("transfer".to_string()),
                source_channel: Some("channel-0".to_string()),
                destination_port: Some("transfer".to_string()),
                destination_channel: Some("channel-1".to_string()),
                timeout_height: Some(RequestPacketTimeoutHeight {
                    revision_height: Some(0u64),
                    revision_number: Some(0u64),
                }),
                data: Some(Binary::from([0; 0])),
                timeout_timestamp: Some(0u64),
            },
        },
    )
    .unwrap();
    assert_eq!(ICA.load(deps.as_mut().storage).unwrap(), IcaState::Timeout);
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-neutron-contracts__drop-pump-sudo-timeout".to_string()
            )
            .add_attributes(vec![
                cosmwasm_std::attr("action".to_string(), "sudo_timeout".to_string()),
                cosmwasm_std::attr("request_id".to_string(), "0".to_string())
            ])
        )
    )
}

#[test]
fn test_sudo_kv_query_result_not_supported() {
    let mut deps = mock_dependencies(&[]);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::KVQueryResult { query_id: 0u64 },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "KVQueryResult and TxQueryResult are not supported".to_string()
        })
    );
}

#[test]
fn test_sudo_tx_query_result_not_supported() {
    let mut deps = mock_dependencies(&[]);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::TxQueryResult {
            query_id: 0u64,
            height: Height {
                revision_height: 0u64,
                revision_number: 0u64,
            },
            data: Binary::from(&[0; 0]),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "KVQueryResult and TxQueryResult are not supported".to_string()
        })
    );
}

#[test]
fn test_sudo_open_ack_invalid_version() {
    let mut deps = mock_dependencies(&[]);
    assert_eq!(ICA.load(deps.as_mut().storage).unwrap(), IcaState::None);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::OpenAck {
            port_id: "transfer".to_string(),
            channel_id: "channel-0".to_string(),
            counterparty_channel_id: "channel-0".to_string(),
            counterparty_version: "invalid_version".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(cosmwasm_std::StdError::generic_err(
            "can't parse version".to_string()
        ))
    );
}

#[test]
fn test_sudo_open_ack() {
    let mut deps = mock_dependencies(&[]);
    assert_eq!(ICA.load(deps.as_mut().storage).unwrap(), IcaState::None);
    let res = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::OpenAck {
            port_id: "transfer".to_string(),
            channel_id: "channel-0".to_string(),
            counterparty_channel_id: "channel-0".to_string(),
            counterparty_version: "{\"version\":\"0\",\"controller_connection_id\":\"0\",\"host_connection_id\":\"0\",\"address\":\"somebody\",\"encoding\":\"something\",\"tx_type\":\"something\"}".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        ICA.load(deps.as_mut().storage).unwrap(),
        IcaState::Registered {
            ica_address: "somebody".to_string(),
            port_id: "transfer".to_string(),
            channel_id: "channel-0".to_string(),
        }
    );
    assert_eq!(res, cosmwasm_std::Response::new());
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    let config = get_default_config();
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let query_res: drop_staking_base::state::pump::Config = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::pump::QueryMsg::Config {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(query_res, config);
}

#[test]
fn test_query_ica() {
    let deps = mock_dependencies(&[]);
    let query_res: drop_helpers::ica::IcaState = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::pump::QueryMsg::Ica {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(query_res, IcaState::None {});
}

#[test]
fn test_query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("admin")).unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::pump::QueryMsg::Ownership {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(cosmwasm_std::Addr::unchecked("admin".to_string())),
            pending_expiry: None,
            pending_owner: None
        }
    );
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: Some(cw_ownable::Expiration::Never {}),
            },
        ),
    )
    .unwrap();
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("new_owner", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::AcceptOwnership {},
        ),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::pump::QueryMsg::Ownership {},
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
fn test_migrate_wrong_contract() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    cw2::set_contract_version(deps_mut.storage, "wrong_contract_name", "0.0.1").unwrap();

    let res = crate::contract::migrate(
        deps.as_mut(),
        mock_env(),
        drop_staking_base::msg::pump::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}
