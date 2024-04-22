use std::vec;

use crate::contract::Puppeteer;
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Binary, Coin, CosmosMsg, DepsMut, Event, Response, StdError, SubMsg,
    Uint128,
};
use drop_helpers::{interchain::IBCFees, testing::mock_dependencies};
use drop_puppeteer_base::state::{PuppeteerBase, ReplyMsg};
use drop_staking_base::state::puppeteer::{Config, KVQueryType};
use drop_staking_base::{msg::puppeteer::InstantiateMsg, state::puppeteer::ConfigOptional};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    sudo::msg::SudoMsg,
    NeutronError,
};
use prost::Message;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        owner: Some("owner".to_string()),
        connection_id: "connection_id".to_string(),
        port_id: "port_id".to_string(),
        update_period: 60u64,
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec!["allowed_sender".to_string()],
        transfer_channel_id: "transfer_channel_id".to_string(),
        sdk_version: "0.45.0".to_string(),
        ibc_fees: IBCFees {
            recv_fee: Uint128::from(3000u128),
            ack_fee: Uint128::from(4000u128),
            timeout_fee: Uint128::from(5000u128),
            register_fee: Uint128::from(6000u128),
        },
    };
    let env = mock_env();
    let res =
        crate::contract::instantiate(deps.as_mut(), env, mock_info("sender", &[]), msg).unwrap();
    assert_eq!(res, Response::new());
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, get_base_config());
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = Puppeteer::default();
    puppeteer_base
        .config
        .save(deps.as_mut().storage, &get_base_config())
        .unwrap();
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::UpdateConfig {
        new_config: ConfigOptional {
            update_period: Some(121u64),
            remote_denom: Some("new_remote_denom".to_string()),
            allowed_senders: Some(vec![Addr::unchecked("new_allowed_sender")]),
            transfer_channel_id: Some("new_transfer_channel_id".to_string()),
            connection_id: Some("new_connection_id".to_string()),
            port_id: Some("new_port_id".to_string()),
            proxy_address: Some(Addr::unchecked("new_proxy_address")),
            sdk_version: Some("0.47.0".to_string()),
        },
    };
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let env = mock_env();
    let res =
        crate::contract::execute(deps.as_mut(), env, mock_info("owner", &[]), msg.clone()).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-neutron-contracts__drop-puppeteer-config_update")
                .add_attributes(vec![
                    ("proxy_address", "new_proxy_address"),
                    ("remote_denom", "new_remote_denom"),
                    ("connection_id", "new_connection_id"),
                    ("port_id", "new_port_id"),
                    ("update_period", "121"),
                    ("allowed_senders", "1"),
                    ("transfer_channel_id", "new_transfer_channel_id"),
                    ("sdk_version", "0.47.0"),
                ])
        )
    );
    let config = puppeteer_base.config.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            port_id: "new_port_id".to_string(),
            connection_id: "new_connection_id".to_string(),
            update_period: 121u64,
            remote_denom: "new_remote_denom".to_string(),
            allowed_senders: vec![Addr::unchecked("new_allowed_sender")],
            transfer_channel_id: "new_transfer_channel_id".to_string(),
            sdk_version: "0.47.0".to_string(),
            proxy_address: Some(Addr::unchecked("new_proxy_address")),
        }
    );
}

#[test]
fn test_execute_delegate() {
    let mut deps = mock_dependencies(&[]);
    let pupeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
        items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
        fee: None,
        reply_to: "some_reply_to".to_string(),
        timeout: Some(100u64),
    };
    let env = mock_env();
    let res = crate::contract::execute(
        deps.as_mut(),
        env,
        mock_info("not_allowed_sender", &[]),
        msg.clone(),
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "Sender is not allowed".to_string()
        })
    );
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        msg,
    )
    .unwrap();
    let msg = cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate {
        delegator_address: "ica_address".to_string(),
        validator_address: "valoper1".to_string(),
        amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: "remote_denom".to_string(),
            amount: "1000".to_string(),
        }),
    };
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    let any_msg = neutron_sdk::bindings::types::ProtobufAny {
        type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        value: Binary::from(buf),
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![any_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        ))
    );
    let tx_state = pupeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        tx_state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::InProgress,
            reply_to: Some("some_reply_to".to_string()),
            transaction: Some(drop_puppeteer_base::msg::Transaction::Delegate {
                interchain_account_id: "DROP".to_string(),
                denom: "remote_denom".to_string(),
                items: vec![("valoper1".to_string(), Uint128::from(1000u128))]
            })
        }
    );
}

#[test]
fn test_execute_grant_delegate() {
    let mut deps = mock_dependencies(&[]);
    let pupeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::GrantDelegate {
        grantee: "grantee".to_string(),
        timeout: Some(100u64),
    };
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        msg.clone(),
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "Sender is not allowed".to_string()
        })
    );
    let env = mock_env();
    let res = crate::contract::execute(deps.as_mut(), env, mock_info("allowed_sender", &[]), msg)
        .unwrap();
    let msg = cosmos_sdk_proto::cosmos::authz::v1beta1::MsgGrant {
        granter: "ica_address".to_string(),
        grantee: "grantee".to_string(),
        grant: Some(cosmos_sdk_proto::cosmos::authz::v1beta1::Grant {
            expiration: Some(prost_types::Timestamp {
                seconds: mock_env()
                    .block
                    .time
                    .plus_days(365 * 120 + 30)
                    .seconds()
                    .try_into()
                    .unwrap(),
                nanos: 0,
            }),
            authorization: Some(cosmos_sdk_proto::Any {
                type_url: "/cosmos.authz.v1beta1.GenericAuthorization".to_string(),
                value: cosmos_sdk_proto::cosmos::authz::v1beta1::GenericAuthorization {
                    msg: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                }
                .encode_to_vec(),
            }),
        }),
    };
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    let any_msg = neutron_sdk::bindings::types::ProtobufAny {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: Binary::from(buf),
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![any_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        ))
    );
    let tx_state = pupeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        tx_state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::InProgress,
            reply_to: Some("".to_string()),
            transaction: Some(drop_puppeteer_base::msg::Transaction::GrantDelegate {
                interchain_account_id: "ica_address".to_string(),
                grantee: "grantee".to_string(),
            })
        }
    );
}

#[test]
fn test_execute_undelegate() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
        batch_id: 0u128,
        items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
        reply_to: "some_reply_to".to_string(),
        timeout: Some(100u64),
    };
    let env = mock_env();
    let res = crate::contract::execute(
        deps.as_mut(),
        env,
        mock_info("not_allowed_sender", &[]),
        msg.clone(),
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "Sender is not allowed".to_string()
        })
    );
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        msg,
    )
    .unwrap();
    let msg = cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate {
        delegator_address: "ica_address".to_string(),
        validator_address: "valoper1".to_string(),
        amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: "remote_denom".to_string(),
            amount: "1000".to_string(),
        }),
    };
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    let any_msg = neutron_sdk::bindings::types::ProtobufAny {
        type_url: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
        value: Binary::from(buf),
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![any_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        ))
    );
    let tx_state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        tx_state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::InProgress,
            reply_to: Some("some_reply_to".to_string()),
            transaction: Some(drop_puppeteer_base::msg::Transaction::Undelegate {
                batch_id: 0u128,
                interchain_account_id: "DROP".to_string(),
                denom: "remote_denom".to_string(),
                items: vec![("valoper1".to_string(), Uint128::from(1000u128))]
            })
        }
    );
}

#[test]
fn test_execute_redeem_share() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
        items: vec![drop_puppeteer_base::state::RedeemShareItem {
            amount: Uint128::from(1000u128),
            remote_denom: "remote_denom".to_string(),
            local_denom: "local_denom".to_string(),
        }],
        timeout: Some(100u64),
        reply_to: "some_reply_to".to_string(),
    };
    let env = mock_env();
    let res = crate::contract::execute(
        deps.as_mut(),
        env,
        mock_info("not_allowed_sender", &[]),
        msg.clone(),
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "Sender is not allowed".to_string()
        })
    );
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        msg,
    )
    .unwrap();
    let msg = crate::proto::liquidstaking::staking::v1beta1::MsgRedeemTokensforShares {
        amount: Some(crate::proto::cosmos::base::v1beta1::Coin {
            denom: "remote_denom".to_string(),
            amount: "1000".to_string(),
        }),
        delegator_address: "ica_address".to_string(),
    };
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    let any_msg = neutron_sdk::bindings::types::ProtobufAny {
        type_url: "/cosmos.staking.v1beta1.MsgRedeemTokensForShares".to_string(),
        value: Binary::from(buf),
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![any_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        )).add_attributes(vec![("action", "redeem_share"), ("items", "[RedeemShareItem { amount: Uint128(1000), remote_denom: \"remote_denom\", local_denom: \"local_denom\" }]")])
    );
    let tx_state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        tx_state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::InProgress,
            reply_to: Some("some_reply_to".to_string()),
            transaction: Some(drop_puppeteer_base::msg::Transaction::RedeemShares {
                interchain_account_id: "DROP".to_string(),
                items: vec![drop_puppeteer_base::state::RedeemShareItem {
                    amount: Uint128::from(1000u128),
                    remote_denom: "remote_denom".to_string(),
                    local_denom: "local_denom".to_string()
                }]
            })
        }
    );
}

#[test]
fn test_execute_set_fees() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::SetFees {
        recv_fee: Uint128::from(3000u128),
        ack_fee: Uint128::from(4000u128),
        timeout_fee: Uint128::from(5000u128),
        register_fee: Uint128::from(6000u128),
    };
    let env = mock_env();
    let res = crate::contract::execute(
        deps.as_mut(),
        env,
        mock_info("not_allowed_sender", &[]),
        msg.clone(),
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner {}
        )
    );
    let res =
        crate::contract::execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();
    assert_eq!(res, Response::new());
    let fees = puppeteer_base.ibc_fee.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        fees,
        IbcFee {
            recv_fee: vec![Coin::new(3000u128, "untrn")],
            ack_fee: vec![Coin::new(4000u128, "untrn")],
            timeout_fee: vec![Coin::new(5000u128, "untrn")],
        }
    );
}

#[test]
fn test_sudo_response_tx_state_wrong() {
    // Test that the contract returns an error if the tx state is not in progress
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = SudoMsg::Response {
        request: neutron_sdk::sudo::msg::RequestPacket {
            sequence: Some(1u64),
            source_port: Some("source_port".to_string()),
            source_channel: Some("source_channel".to_string()),
            destination_port: Some("destination_port".to_string()),
            destination_channel: Some("destination_channel".to_string()),
            data: None,
            timeout_height: None,
            timeout_timestamp: None,
        },
        data: Binary::from(vec![]),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::Idle,
                reply_to: None,
                transaction: None,
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg);
    assert_eq!(
        res.unwrap_err(),
        NeutronError::Std(StdError::generic_err(
            "Transaction txState is not equal to expected: WaitingForAck"
        ))
    );
}

#[test]
fn test_sudo_response_ok() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let request = neutron_sdk::sudo::msg::RequestPacket {
        sequence: Some(1u64),
        source_port: Some("source_port".to_string()),
        source_channel: Some("source_channel".to_string()),
        destination_port: Some("destination_port".to_string()),
        destination_channel: Some("destination_channel".to_string()),
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    };
    let transaction = drop_puppeteer_base::msg::Transaction::IBCTransfer {
        denom: "remote_denom".to_string(),
        amount: 1000u128,
        recipient: "recipient".to_string(),
        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
    };
    let msg = SudoMsg::Response {
        request: request.clone(),
        data: Binary::from(vec![]),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: Some("reply_to_contract".to_string()),
                transaction: Some(transaction.clone()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_message(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: "reply_to_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::PuppeteerHook(
                    Box::new(drop_puppeteer_base::msg::ResponseHookMsg::Success(
                        drop_puppeteer_base::msg::ResponseHookSuccessMsg {
                            request_id: 1,
                            request,
                            transaction,
                            answers: vec![drop_puppeteer_base::msg::ResponseAnswer::IBCTransfer(
                                drop_puppeteer_base::proto::MsgIBCTransfer {}
                            )]
                        }
                    ))
                ))
                .unwrap(),
                funds: vec![]
            }))
            .add_event(
                Event::new("puppeteer-sudo-response")
                    .add_attributes(vec![("action", "sudo_response"), ("request_id", "1")])
            )
    );
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        ica,
        drop_helpers::ica::IcaState::Registered {
            ica_address: "ica_address".to_string()
        }
    );
    let state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::Idle,
            reply_to: None,
            transaction: None,
        }
    );
}

#[test]
fn test_sudo_response_error() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let request = neutron_sdk::sudo::msg::RequestPacket {
        sequence: Some(1u64),
        source_port: Some("source_port".to_string()),
        source_channel: Some("source_channel".to_string()),
        destination_port: Some("destination_port".to_string()),
        destination_channel: Some("destination_channel".to_string()),
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    };
    let transaction = drop_puppeteer_base::msg::Transaction::IBCTransfer {
        denom: "remote_denom".to_string(),
        amount: 1000u128,
        recipient: "recipient".to_string(),
        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
    };
    let msg = SudoMsg::Error {
        request: request.clone(),
        details: "some shit happened".to_string(),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: Some("reply_to_contract".to_string()),
                transaction: Some(transaction.clone()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_message(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: "reply_to_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::PuppeteerHook(
                    Box::new(drop_puppeteer_base::msg::ResponseHookMsg::Error(
                        drop_puppeteer_base::msg::ResponseHookErrorMsg {
                            request_id: 1,
                            request,
                            transaction,
                            details: "some shit happened".to_string()
                        }
                    ))
                ))
                .unwrap(),
                funds: vec![]
            }))
            .add_event(Event::new("puppeteer-sudo-error").add_attributes(vec![
                ("action", "sudo_error"),
                ("request_id", "1"),
                ("details", "some shit happened")
            ]))
    );
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        ica,
        drop_helpers::ica::IcaState::Registered {
            ica_address: "ica_address".to_string()
        }
    );
    let state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::Idle,
            reply_to: None,
            transaction: None,
        }
    );
}

#[test]
fn test_sudo_open_ack() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = SudoMsg::OpenAck {
        port_id: "port_id_1".to_string(),
        channel_id: "channel_1".to_string(),
        counterparty_channel_id: "counterparty_channel_id_1".to_string(),
        counterparty_version: "{\"version\": \"1\",\"controller_connection_id\": \"connection_id\",\"host_connection_id\": \"host_connection_id\",\"address\": \"ica_address\",\"encoding\": \"amino\",\"tx_type\": \"cosmos-sdk/MsgSend\"}".to_string(),
    };
    let env = mock_env();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(res, Response::new());
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        ica,
        drop_helpers::ica::IcaState::Registered {
            ica_address: "ica_address".to_string()
        }
    );
}

#[test]
fn test_sudo_response_timeout() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let request = neutron_sdk::sudo::msg::RequestPacket {
        sequence: Some(1u64),
        source_port: Some("source_port".to_string()),
        source_channel: Some("source_channel".to_string()),
        destination_port: Some("destination_port".to_string()),
        destination_channel: Some("destination_channel".to_string()),
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    };
    let transaction = drop_puppeteer_base::msg::Transaction::IBCTransfer {
        denom: "remote_denom".to_string(),
        amount: 1000u128,
        recipient: "recipient".to_string(),
        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
    };
    let msg = SudoMsg::Timeout {
        request: request.clone(),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: Some("reply_to_contract".to_string()),
                transaction: Some(transaction.clone()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_message(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: "reply_to_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::PuppeteerHook(
                    Box::new(drop_puppeteer_base::msg::ResponseHookMsg::Error(
                        drop_puppeteer_base::msg::ResponseHookErrorMsg {
                            request_id: 1,
                            request,
                            transaction,
                            details: "Timeout".to_string()
                        }
                    ))
                ))
                .unwrap(),
                funds: vec![]
            }))
            .add_event(
                Event::new("puppeteer-sudo-timeout")
                    .add_attributes(vec![("action", "sudo_timeout"), ("request_id", "1"),])
            )
    );
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(ica, drop_helpers::ica::IcaState::Timeout);
    let state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::Idle,
            reply_to: None,
            transaction: None,
        }
    );
}

fn get_base_config() -> Config {
    Config {
        port_id: "port_id".to_string(),
        connection_id: "connection_id".to_string(),
        update_period: 60u64,
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec![Addr::unchecked("allowed_sender")],
        transfer_channel_id: "transfer_channel_id".to_string(),
        sdk_version: "0.45.0".to_string(),
        proxy_address: None,
    }
}

fn base_init(
    deps_mut: &mut DepsMut<NeutronQuery>,
) -> PuppeteerBase<'static, drop_staking_base::state::puppeteer::Config, KVQueryType> {
    let puppeteer_base = Puppeteer::default();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    puppeteer_base
        .config
        .save(deps_mut.storage, &get_base_config())
        .unwrap();
    puppeteer_base
        .ica
        .set_address(deps_mut.storage, "ica_address")
        .unwrap();
    puppeteer_base
        .ibc_fee
        .save(deps_mut.storage, &get_standard_fees())
        .unwrap();
    puppeteer_base
}

fn get_standard_fees() -> IbcFee {
    IbcFee {
        recv_fee: vec![Coin::new(1000u128, "untrn")],
        ack_fee: vec![Coin::new(2000u128, "untrn")],
        timeout_fee: vec![Coin::new(3000u128, "untrn")],
    }
}
