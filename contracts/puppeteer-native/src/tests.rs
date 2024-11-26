use cosmwasm_schema::schemars;
use cosmwasm_std::{
    coin, coins, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal256, DepsMut, Event, Response, StdError,
    SubMsg, Timestamp, Uint128, Uint64,
};
use drop_helpers::{
    ibc_client_state::{
        ChannelClientStateResponse, ClientState, Fraction, Height, IdentifiedClientState,
    },
    testing::mock_dependencies,
};

use drop_puppeteer_base::state::{BalancesAndDelegationsState, PuppeteerBase, ReplyMsg};
use drop_staking_base::state::{
    puppeteer::{BalancesAndDelegations, Delegations, DropDelegation, KVQueryType},
    puppeteer_native::{Config, ConfigOptional, CONFIG},
};
use drop_staking_base::{
    msg::puppeteer_native::InstantiateMsg, state::puppeteer::NON_NATIVE_REWARD_BALANCES,
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::{NeutronQuery, QueryRegisteredQueryResultResponse},
        types::{InterchainQueryResult, StorageValue},
    },
    interchain_queries::v045::types::Balances,
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::SudoMsg,
    NeutronError,
};
use prost::Message;
use schemars::_serde_json::to_string;

use std::vec;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        owner: Some("owner".to_string()),
        native_bond_provider: "native_bond_provider".to_string(),
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec!["allowed_sender".to_string()],
    };
    let env = mock_env();
    let res =
        crate::contract::instantiate(deps.as_mut(), env, mock_info("sender", &[]), msg).unwrap();
    assert_eq!(res, Response::new());
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, get_base_config());
    assert_eq!(
        cosmwasm_std::Addr::unchecked("owner"),
        cw_ownable::get_ownership(deps.as_mut().storage)
            .unwrap()
            .owner
            .unwrap()
    );
}

#[test]
fn test_execute_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_base_config())
        .unwrap();
    let msg = drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateConfig {
        new_config: ConfigOptional {
            remote_denom: Some("new_remote_denom".to_string()),
            native_bond_provider: Some(Addr::unchecked("native_bond_provider")),
            allowed_senders: Some(vec!["new_allowed_sender".to_string()]),
        },
    };
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    )
}

#[test]
fn test_execute_update_config() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_base_config())
        .unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                remote_denom: Some("new_remote_denom".to_string()),
                native_bond_provider: Some(Addr::unchecked("native_bond_provider")),
                allowed_senders: Some(vec!["new_allowed_sender".to_string()]),
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-neutron-contracts__drop-puppeteer-config_update")
                .add_attributes(vec![
                    ("remote_denom", "new_remote_denom"),
                    ("allowed_senders", "1"),
                    ("native_bond_provider", "native_bond_provider"),
                ])
        )
    );

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            native_bond_provider: Addr::unchecked("native_bond_provider"),
            remote_denom: "new_remote_denom".to_string(),
            allowed_senders: vec![Addr::unchecked("new_allowed_sender")],
        }
    );
}

#[test]
fn test_execute_setup_protocol_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::SetupProtocol {
            rewards_withdraw_address: "rewards_withdraw_address".to_string(),
        },
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_setup_protocol() {
    let mut deps = mock_dependencies(&[]);

    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::SetupProtocol {
            rewards_withdraw_address: "rewards_withdraw_address".to_string(),
        },
    )
    .unwrap();

    let distribution_msg = {
        neutron_sdk::bindings::types::ProtobufAny {
            type_url: "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
            value: Binary::from(
                cosmos_sdk_proto::cosmos::distribution::v1beta1::MsgSetWithdrawAddress {
                    delegator_address: "ica_address".to_string(),
                    withdraw_address: "rewards_withdraw_address".to_string(),
                }
                .encode_to_vec(),
            ),
        }
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![distribution_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        ))
    );
}

#[test]
fn test_execute_undelegate_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Undelegate {
            batch_id: 0u128,
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
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
    base_init(&mut deps.as_mut());

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Undelegate {
            batch_id: 0u128,
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();

    let undelegate_msg = drop_helpers::interchain::prepare_any_msg(
        cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate {
            delegator_address: "ica_address".to_string(),
            validator_address: "valoper1".to_string(),
            amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                denom: "remote_denom".to_string(),
                amount: "1000".to_string(),
            }),
        },
        "/cosmos.staking.v1beta1.MsgUndelegate",
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![undelegate_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        ))
    );
}

#[test]
fn test_execute_claim_rewards_and_optionaly_transfer_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators: vec!["validator1".to_string(), "validator2".to_string()],
            transfer: Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg {
                batch_ids: vec![0u128, 1u128, 2u128],
                emergency: true,
                amount: Uint128::from(123u64),
                recipient: "some_recipient".to_string(),
            }),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_claim_rewards_and_optionaly_transfer() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators: vec!["validator1".to_string(), "validator2".to_string()],
            transfer: Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg {
                batch_ids: vec![0u128, 1u128, 2u128],
                emergency: true,
                amount: Uint128::from(123u64),
                recipient: "some_recipient".to_string(),
            }),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();
    let ica_address = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 65536u64,
            msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![
                    drop_helpers::interchain::prepare_any_msg(
                        cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
                            from_address: ica_address.clone(),
                            to_address: "some_recipient".to_string(),
                            amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                amount: "123".to_string(),
                                denom: puppeteer_base
                                    .config
                                    .load(deps.as_mut().storage)
                                    .unwrap()
                                    .remote_denom
                            }]
                        },
                        "/cosmos.bank.v1beta1.MsgSend",
                    )
                    .unwrap(),
                    drop_helpers::interchain::prepare_any_msg(
                        drop_proto::proto::liquidstaking::distribution::v1beta1::MsgWithdrawDelegatorReward {
                            delegator_address: ica_address.clone(),
                            validator_address: "validator1".to_string(),
                        },
                        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward",
                    )
                    .unwrap(),
                    drop_helpers::interchain::prepare_any_msg(
                        drop_proto::proto::liquidstaking::distribution::v1beta1::MsgWithdrawDelegatorReward {
                            delegator_address: ica_address.clone(),
                            validator_address: "validator2".to_string(),
                        },
                        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward",
                    )
                    .unwrap()
                ],
                "".to_string(),
                100u64,
                IbcFee {
                    recv_fee: vec![],
                    ack_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(100u64),
                    }],
                    timeout_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(200u64),
                    }],
                },
            )),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success
        }),
    );
}

fn get_base_config() -> Config {
    Config {
        native_bond_provider: Addr::unchecked("native_bond_provider"),
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec![Addr::unchecked("allowed_sender")],
    }
}

fn base_init(deps_mut: &mut DepsMut<NeutronQuery>) {
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG.save(deps_mut.storage, &get_base_config()).unwrap();
}

fn get_standard_fees() -> IbcFee {
    IbcFee {
        recv_fee: vec![],
        ack_fee: coins(100, "untrn"),
        timeout_fee: coins(200, "untrn"),
    }
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: Some(cw_ownable::Expiration::Never {}),
            },
        ),
    )
    .unwrap();
    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("new_owner", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::AcceptOwnership {},
        ),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Ownership {},
            },
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
fn test_query_extension_delegations_none() {
    let deps = mock_dependencies(&[]);
    let query_res: drop_staking_base::msg::puppeteer::DelegationsResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Delegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::DelegationsResponse {
            delegations: Delegations {
                delegations: vec![],
            },
            remote_height: 0,
            local_height: 0,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_extension_delegations_some() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    puppeteer_base
        .last_complete_delegations_and_balances_key
        .save(deps.as_mut().storage, &0u64)
        .unwrap();
    let delegations = vec![
        DropDelegation {
            delegator: Addr::unchecked("delegator1"),
            validator: "validator1".to_string(),
            amount: cosmwasm_std::Coin::new(100, "denom1"),
            share_ratio: Decimal256::from_ratio(
                cosmwasm_std::Uint256::from(0u64),
                cosmwasm_std::Uint256::from(1u64),
            ),
        },
        DropDelegation {
            delegator: Addr::unchecked("delegator2"),
            validator: "validator2".to_string(),
            amount: cosmwasm_std::Coin::new(100, "denom2"),
            share_ratio: Decimal256::from_ratio(
                cosmwasm_std::Uint256::from(0u64),
                cosmwasm_std::Uint256::from(1u64),
            ),
        },
    ];
    puppeteer_base
        .delegations_and_balances
        .save(
            deps.as_mut().storage,
            &0u64,
            &BalancesAndDelegationsState {
                data: BalancesAndDelegations {
                    balances: Balances { coins: vec![] },
                    delegations: Delegations {
                        delegations: delegations.clone(),
                    },
                },
                remote_height: 123u64,
                local_height: 123u64,
                timestamp: Timestamp::default(),
                collected_chunks: vec![],
            },
        )
        .unwrap();
    let query_res: drop_staking_base::msg::puppeteer::DelegationsResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::DelegationsResponse {
            delegations: Delegations { delegations },
            remote_height: 123u64,
            local_height: 123u64,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_extension_balances_none() {
    let deps = mock_dependencies(&[]);
    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Balances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances { coins: vec![] },
            remote_height: 0,
            local_height: 0,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_extension_balances_some() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    puppeteer_base
        .last_complete_delegations_and_balances_key
        .save(deps.as_mut().storage, &0u64)
        .unwrap();
    let coins = vec![
        cosmwasm_std::Coin::new(123u128, "denom1".to_string()),
        cosmwasm_std::Coin::new(123u128, "denom2".to_string()),
    ];
    puppeteer_base
        .delegations_and_balances
        .save(
            deps.as_mut().storage,
            &0u64,
            &BalancesAndDelegationsState {
                data: BalancesAndDelegations {
                    balances: Balances {
                        coins: coins.clone(),
                    },
                    delegations: Delegations {
                        delegations: vec![],
                    },
                },
                remote_height: 123u64,
                local_height: 123u64,
                timestamp: Timestamp::default(),
                collected_chunks: vec![],
            },
        )
        .unwrap();
    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances { coins },
            remote_height: 123u64,
            local_height: 123u64,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_non_native_rewards_balances() {
    let mut deps = mock_dependencies(&[]);
    let coins = vec![
        cosmwasm_std::Coin::new(123u128, "denom1".to_string()),
        cosmwasm_std::Coin::new(123u128, "denom2".to_string()),
    ];
    NON_NATIVE_REWARD_BALANCES
        .save(
            deps.as_mut().storage,
            &BalancesAndDelegationsState {
                data: drop_staking_base::msg::puppeteer::MultiBalances {
                    coins: coins.clone(),
                },
                remote_height: 1u64,
                local_height: 2u64,
                timestamp: Timestamp::default(),
                collected_chunks: vec![],
            },
        )
        .unwrap();
    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::NonNativeRewardsBalances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances { coins },
            remote_height: 1u64,
            local_height: 2u64,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_unbonding_delegations() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    let unbonding_delegations = vec![
        drop_puppeteer_base::state::UnbondingDelegation {
            validator_address: "validator_address1".to_string(),
            query_id: 1u64,
            unbonding_delegations: vec![
                neutron_sdk::interchain_queries::v047::types::UnbondingEntry {
                    balance: Uint128::from(0u64),
                    completion_time: None,
                    creation_height: 0u64,
                    initial_balance: Uint128::from(0u64),
                },
            ],
            last_updated_height: 0u64,
        },
        drop_puppeteer_base::state::UnbondingDelegation {
            validator_address: "validator_address2".to_string(),
            query_id: 2u64,
            unbonding_delegations: vec![],
            last_updated_height: 0u64,
        },
    ];
    puppeteer_base
        .unbonding_delegations
        .save(deps.as_mut().storage, "key1", &unbonding_delegations[0])
        .unwrap();
    puppeteer_base
        .unbonding_delegations
        .save(deps.as_mut().storage, "key2", &unbonding_delegations[1])
        .unwrap();
    let query_res: Vec<drop_puppeteer_base::state::UnbondingDelegation> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::UnbondingDelegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(query_res, unbonding_delegations);
}
