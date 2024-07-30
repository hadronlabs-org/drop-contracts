use crate::contract::Puppeteer;
use cosmwasm_schema::schemars;
use cosmwasm_std::{
    coin, coins, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Binary, CosmosMsg, Delegation, DepsMut, Event, Response, StdError,
    SubMsg, Timestamp, Uint128, Uint64,
};
use drop_helpers::{
    ibc_client_state::{
        ChannelClientStateResponse, ClientState, Fraction, Height, IdentifiedClientState,
    },
    testing::mock_dependencies,
};
use drop_puppeteer_base::state::{BalancesAndDelegationsState, PuppeteerBase, ReplyMsg};
use drop_staking_base::{
    msg::puppeteer::{BalancesAndDelegations, InstantiateMsg},
    state::puppeteer::{Config, ConfigOptional, KVQueryType, DELEGATIONS_AND_BALANCE},
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::{NeutronQuery, QueryRegisteredQueryResultResponse},
        types::{InterchainQueryResult, StorageValue},
    },
    interchain_queries::v045::types::{Balances, Delegations},
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::SudoMsg,
    NeutronError,
};
use prost::Message;
use schemars::_serde_json::to_string;

use std::vec;

fn build_interchain_query_response() -> Binary {
    let res: Vec<StorageValue> = from_json(
        r#"[
            {"storage_prefix":"bank","key":"AiCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZnN0YWtl","value":"MjgwODc4MzI4","Proof":null},{"storage_prefix":"staking","key":"UQ==","value":"CgMImCoQZBgHIJBOKgVzdGFrZTIBMDoVNTAwMDAwMDAwMDAwMDAwMDAwMDAwQhMxMDAwMDAwMDAwMDAwMDAwMDAwShMxMDAwMDAwMDAwMDAwMDAwMDAw","Proof":null},{"storage_prefix":"staking","key":"MSCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZhQc2kl1CUPnDLfq4SkyqZCEz9lSeg==","value":"CkFjb3Ntb3MxbnVqeTN2bDNyd3czY3k4dGY4cGRydTVqcDNmOXBwbWthZHdzNTUzY2szcXJ5ZzJ0amFucXQzOXhudhI0Y29zbW9zdmFsb3BlcjFybmR5amFnZmcwbnNlZGwydXk1bjkydnNzbjhhajVuNjd0MG5meBodMTM0NDIwMjU5ODgwMDAwMDAwMDAwMDAwMDAwMDA=","Proof":null},{"storage_prefix":"staking","key":"IRQc2kl1CUPnDLfq4SkyqZCEz9lSeg==","value":"CjRjb3Ntb3N2YWxvcGVyMXJuZHlqYWdmZzBuc2VkbDJ1eTVuOTJ2c3NuOGFqNW42N3QwbmZ4EkMKHS9jb3Ntb3MuY3J5cHRvLmVkMjU1MTkuUHViS2V5EiIKIB4wQBwhrBNl3pDg7PIpHeuQVlEhXOPPKLVWN7JJSp+MIAMqCzI1MDczNzQyMzI5Mh0yNTA3Mzc0MjMyOTAwMDAwMDAwMDAwMDAwMDAwMDoKCgh2YWxnYWlhMUoAUkoKOwoSMTAwMDAwMDAwMDAwMDAwMDAwEhIyMDAwMDAwMDAwMDAwMDAwMDAaETEwMDAwMDAwMDAwMDAwMDAwEgsIn/DYsgYQtvORQFoBMHIcMTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMHodMjQwNzM3NDIyNTQwMDAwMDAwMDAwMDAwMDAwMDA=","Proof":null},{"storage_prefix":"staking","key":"MSCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZhRF6sE4roJR9V4+ACeV5W/dXDFvzA==","value":"CkFjb3Ntb3MxbnVqeTN2bDNyd3czY3k4dGY4cGRydTVqcDNmOXBwbWthZHdzNTUzY2szcXJ5ZzJ0amFucXQzOXhudhI0Y29zbW9zdmFsb3BlcjFnaDR2enc5d3NmZ2wyaDM3cXFuZXRldDBtNHdyem03djd4M2o5eBodMTM0NDIwMjU5ODgwMDAwMDAwMDAwMDAwMDAwMDA=","Proof":null},{"storage_prefix":"staking","key":"IRRF6sE4roJR9V4+ACeV5W/dXDFvzA==","value":"CjRjb3Ntb3N2YWxvcGVyMWdoNHZ6dzl3c2ZnbDJoMzdxcW5ldGV0MG00d3J6bTd2N3gzajl4EkMKHS9jb3Ntb3MuY3J5cHRvLmVkMjU1MTkuUHViS2V5EiIKIBBp8oY6i7JDYLibq1ifiAsifWSXqMG943z+kPiorWjhIAMqCzI1MDczNzM0ODU2Mh0yNTA3MzczNDg1NjAwMDAwMDAwMDAwMDAwMDAwMDoKCgh2YWxnYWlhMEoAUkoKOwoSMTAwMDAwMDAwMDAwMDAwMDAwEhIyMDAwMDAwMDAwMDAwMDAwMDAaETEwMDAwMDAwMDAwMDAwMDAwEgsIn/DYsgYQtvORQFoBMHIcMTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMHodMjQwNzM3MzQ4NTUwMDAwMDAwMDAwMDAwMDAwMDA=","Proof":null}
        ]"#,
    ).unwrap();

    Binary::from(
        to_string(&QueryRegisteredQueryResultResponse {
            result: InterchainQueryResult {
                kv_results: res,
                height: 123456,
                revision: 2,
            },
        })
        .unwrap()
        .as_bytes(),
    )
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        delegations_queries_chunk_size: Some(2u32),
        owner: Some("owner".to_string()),
        connection_id: "connection_id".to_string(),
        port_id: "port_id".to_string(),
        update_period: 60u64,
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec!["allowed_sender".to_string()],
        transfer_channel_id: "transfer_channel_id".to_string(),
        sdk_version: "0.47.10".to_string(),
        timeout: 100u64,
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
            allowed_senders: Some(vec!["new_allowed_sender".to_string()]),
            transfer_channel_id: Some("new_transfer_channel_id".to_string()),
            connection_id: Some("new_connection_id".to_string()),
            port_id: Some("new_port_id".to_string()),
            sdk_version: Some("0.47.0".to_string()),
            timeout: Some(101u64),
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
                    ("remote_denom", "new_remote_denom"),
                    ("connection_id", "new_connection_id"),
                    ("port_id", "new_port_id"),
                    ("update_period", "121"),
                    ("allowed_senders", "1"),
                    ("transfer_channel_id", "new_transfer_channel_id"),
                    ("sdk_version", "0.47.0"),
                    ("timeout", "101"),
                ])
        )
    );
    let config = puppeteer_base.config.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            delegations_queries_chunk_size: 2u32,
            port_id: "new_port_id".to_string(),
            connection_id: "new_connection_id".to_string(),
            update_period: 121u64,
            remote_denom: "new_remote_denom".to_string(),
            allowed_senders: vec![Addr::unchecked("new_allowed_sender")],
            transfer_channel_id: "new_transfer_channel_id".to_string(),
            sdk_version: "0.47.0".to_string(),
            timeout: 101u64,
        }
    );
}

#[test]
fn test_execute_delegate() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
        items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
        fee: None,
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
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
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
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::GrantDelegate {
        grantee: "grantee".to_string(),
    };
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        msg.clone(),
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
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
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
        batch_id: 0u128,
        items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
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
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
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
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
        items: vec![drop_puppeteer_base::state::RedeemShareItem {
            amount: Uint128::from(1000u128),
            remote_denom: "remote_denom".to_string(),
            local_denom: "local_denom".to_string(),
        }],
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
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
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
fn test_sudo_kv_query_result() {
    let mut deps = mock_dependencies(&[]);

    let query_id = 1u64;

    deps.querier
        .add_query_response(query_id, build_interchain_query_response());

    let puppeteer_base = base_init(&mut deps.as_mut());

    let msg = SudoMsg::KVQueryResult { query_id };
    let env = mock_env();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            query_id,
            &KVQueryType::DelegationsAndBalance {},
        )
        .unwrap();

    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(res, Response::new());

    let state = DELEGATIONS_AND_BALANCE.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        BalancesAndDelegationsState {
            data: BalancesAndDelegations {
                balances: Balances {
                    coins: vec![coin(280878328, "stake")]
                },
                delegations: Delegations {
                    delegations: vec![
                        Delegation {
                            delegator: Addr::unchecked(
                                "cosmos1nujy3vl3rww3cy8tf8pdru5jp3f9ppmkadws553ck3qryg2tjanqt39xnv"
                            ),
                            validator: "cosmosvaloper1rndyjagfg0nsedl2uy5n92vssn8aj5n67t0nfx"
                                .to_string(),
                            amount: coin(13442025988, "stake")
                        },
                        Delegation {
                            delegator: Addr::unchecked(
                                "cosmos1nujy3vl3rww3cy8tf8pdru5jp3f9ppmkadws553ck3qryg2tjanqt39xnv"
                            ),
                            validator: "cosmosvaloper1gh4vzw9wsfgl2h37qqnetet0m4wrzm7v7x3j9x"
                                .to_string(),
                            amount: coin(13442025988, "stake")
                        }
                    ]
                }
            },
            remote_height: 123456,
            local_height: 12345,
            timestamp: Timestamp::from_nanos(1571797419879305533)
        }
    );
}

#[test]
fn test_sudo_response_ok() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.add_stargate_query_response(
        "/ibc.core.channel.v1.Query/ChannelClientState",
        |_data| {
            to_json_binary(&ChannelClientStateResponse {
                identified_client_state: Some(IdentifiedClientState {
                    client_id: "07-tendermint-0".to_string(),
                    client_state: ClientState {
                        chain_id: "test-1".to_string(),
                        type_url: "type_url".to_string(),
                        trust_level: Fraction {
                            numerator: Uint64::from(1u64),
                            denominator: Uint64::from(3u64),
                        },
                        trusting_period: Some("1000".to_string()),
                        unbonding_period: Some("1500".to_string()),
                        max_clock_drift: Some("1000".to_string()),
                        frozen_height: None,
                        latest_height: Some(Height {
                            revision_number: Uint64::from(0u64),
                            revision_height: Uint64::from(54321u64),
                        }),
                        proof_specs: vec![],
                        upgrade_path: vec![],
                        allow_update_after_expiry: true,
                        allow_update_after_misbehaviour: true,
                    },
                }),
                proof: None,
                proof_height: Height {
                    revision_number: Uint64::from(0u64),
                    revision_height: Uint64::from(33333u64),
                },
            })
            .unwrap()
        },
    );

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
                            local_height: 12345,
                            remote_height: 54321,
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
            ica_address: "ica_address".to_string(),
            port_id: "port".to_string(),
            channel_id: "channel".to_string(),
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
            ica_address: "ica_address".to_string(),
            port_id: "port".to_string(),
            channel_id: "channel".to_string(),
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
            ica_address: "ica_address".to_string(),
            port_id: "port_id_1".to_string(),
            channel_id: "channel_1".to_string(),
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

mod register_delegations_and_balance_query {
    use cosmwasm_std::{testing::MockApi, MemoryStorage, OwnedDeps, StdResult};
    use drop_helpers::testing::WasmMockQuerier;
    use drop_puppeteer_base::error::ContractError;

    use super::*;

    fn setup(
        owner: Option<&str>,
    ) -> (
        OwnedDeps<MemoryStorage, MockApi, WasmMockQuerier, NeutronQuery>,
        PuppeteerBase<'static, drop_staking_base::state::puppeteer::Config, KVQueryType>,
    ) {
        let mut deps = mock_dependencies(&[]);
        let puppeteer_base = base_init(&mut deps.as_mut());
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(
            deps_mut.storage,
            deps_mut.api,
            Some(Addr::unchecked(owner.unwrap_or("owner")).as_ref()),
        )
        .unwrap();
        (deps, puppeteer_base)
    }

    #[test]
    fn non_owner() {
        let (mut deps, _puppeteer_base) = setup(None);
        let env = mock_env();
        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery { validators: vec![] } ;
        let res = crate::contract::execute(deps.as_mut(), env, mock_info("not_owner", &[]), msg);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
        );
    }

    #[test]
    fn too_many_validators() {
        let (mut deps, _puppeteer_base) = setup(None);
        let env = mock_env();
        let mut validators = vec![];
        for i in 0..=65536u32 {
            validators.push(format!("valoper{}", i));
        }

        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
            validators
        };
        let res = crate::contract::execute(deps.as_mut(), env, mock_info("owner", &[]), msg);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            ContractError::Std(StdError::generic_err("Too many validators provided"))
        );
    }

    #[test]
    fn happy_path_validators_count_less_than_chunk_size() {
        let (mut deps, puppeteer_base) =
            setup(Some("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"));
        let env = mock_env();
        let validators = vec![
            "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
            "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
        ];
        puppeteer_base
            .ica
            .set_address(
                deps.as_mut().storage,
                "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud",
                "port",
                "channel",
            )
            .unwrap();

        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
            validators
        };
        let res = crate::contract::execute(
            deps.as_mut(),
            env,
            mock_info("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2", &[]),
            msg,
        )
        .unwrap();
        assert_eq!(
            res,
            Response::new().add_submessages(vec![SubMsg::reply_on_success(
                drop_helpers::icq::new_delegations_and_balance_query_msg(
                    "connection_id".to_string(),
                    "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud".to_string(),
                    "remote_denom".to_string(),
                    vec![
                        "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
                        "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
                    ],
                    60,
                    "0.45.0",
                )
                .unwrap(),
                ReplyMsg::KvDelegationsAndBalance { i: 0 }.to_reply_id(),
            )])
        );
    }

    #[test]
    fn happy_path_validators_count_more_than_chunk_size() {
        let (mut deps, puppeteer_base) =
            setup(Some("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"));
        let env = mock_env();
        let validators = vec![
            "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
            "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
            "cosmos15tuf2ewxle6jj6eqd4jm579vpahydzwdsvkrhn".to_string(),
        ];
        puppeteer_base
            .ica
            .set_address(
                deps.as_mut().storage,
                "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud",
                "port",
                "channel",
            )
            .unwrap();
        puppeteer_base
            .delegations_and_balances_query_id_chunk
            .save(deps.as_mut().storage, 1, &2)
            .unwrap();
        puppeteer_base
            .delegations_and_balances_query_id_chunk
            .save(deps.as_mut().storage, 2, &3)
            .unwrap();
        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
            validators
        };
        let res = crate::contract::execute(
            deps.as_mut(),
            env,
            mock_info("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2", &[]),
            msg,
        )
        .unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_messages(vec![
                    NeutronMsg::remove_interchain_query(1),
                    NeutronMsg::remove_interchain_query(2)
                ])
                .add_submessages(vec![
                    SubMsg::reply_on_success(
                        drop_helpers::icq::new_delegations_and_balance_query_msg(
                            "connection_id".to_string(),
                            "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud".to_string(),
                            "remote_denom".to_string(),
                            vec![
                                "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
                                "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
                            ],
                            60,
                            "0.45.0",
                        )
                        .unwrap(),
                        ReplyMsg::KvDelegationsAndBalance { i: 0 }.to_reply_id(),
                    ),
                    SubMsg::reply_on_success(
                        drop_helpers::icq::new_delegations_and_balance_query_msg(
                            "connection_id".to_string(),
                            "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud".to_string(),
                            "remote_denom".to_string(),
                            vec!["cosmos15tuf2ewxle6jj6eqd4jm579vpahydzwdsvkrhn".to_string(),],
                            60,
                            "0.45.0",
                        )
                        .unwrap(),
                        ReplyMsg::KvDelegationsAndBalance { i: 1 }.to_reply_id(),
                    )
                ])
        );
        assert_eq!(
            puppeteer_base
                .delegations_and_balances_query_id_chunk
                .keys(
                    deps.as_ref().storage,
                    None,
                    None,
                    cosmwasm_std::Order::Ascending
                )
                .collect::<StdResult<Vec<u64>>>()
                .unwrap()
                .len(),
            0
        )
    }
}

fn get_base_config() -> Config {
    Config {
        delegations_queries_chunk_size: 2u32,
        port_id: "port_id".to_string(),
        connection_id: "connection_id".to_string(),
        update_period: 60u64,
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec![Addr::unchecked("allowed_sender")],
        transfer_channel_id: "transfer_channel_id".to_string(),
        sdk_version: "0.47.10".to_string(),
        timeout: 100u64,
    }
}

fn base_init(deps_mut: &mut DepsMut<NeutronQuery>) -> PuppeteerBase<'static, Config, KVQueryType> {
    let puppeteer_base = Puppeteer::default();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    puppeteer_base
        .config
        .save(deps_mut.storage, &get_base_config())
        .unwrap();
    puppeteer_base
        .ica
        .set_address(deps_mut.storage, "ica_address", "port", "channel")
        .unwrap();
    puppeteer_base
}

fn get_standard_fees() -> IbcFee {
    IbcFee {
        recv_fee: vec![],
        ack_fee: coins(100, "untrn"),
        timeout_fee: coins(200, "untrn"),
    }
}
