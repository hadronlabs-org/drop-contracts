use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use cosmos_sdk_proto::traits::MessageExt;
use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, CosmosMsg, Event, Response, SubMsg, Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::staker::{
    Config, ConfigOptional, CONFIG, ICA, NON_STAKED_BALANCE, TX_STATE,
};
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    query::min_ibc_fee::MinIbcFeeResponse,
};

fn get_default_config() -> Config {
    Config {
        connection_id: "connection".to_string(),
        timeout: 10u64,
        port_id: "port_id".to_string(),
        transfer_channel_id: "transfer_channel_id".to_string(),
        remote_denom: "remote_denom".to_string(),
        base_denom: "base_denom".to_string(),
        allowed_senders: vec![Addr::unchecked("core")],
        puppeteer_ica: Some("puppeteer_ica".to_string()),
        min_ibc_transfer: Uint128::from(10000u128),
        min_staking_amount: Uint128::from(10000u128),
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = drop_staking_base::msg::staker::InstantiateMsg {
        connection_id: "connection".to_string(),
        timeout: 10u64,
        port_id: "port_id".to_string(),
        transfer_channel_id: "transfer_channel_id".to_string(),
        remote_denom: "remote_denom".to_string(),
        base_denom: "base_denom".to_string(),
        allowed_senders: vec!["core".to_string()],
        min_ibc_transfer: Uint128::from(10000u128),
        min_staking_amount: Uint128::from(10000u128),
        owner: Some("owner".to_string()),
    };
    let res = instantiate(deps.as_mut(), mock_env(), mock_info("admin", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-neutron-contracts__drop-staker-instantiate"
        ).add_attributes(vec![
            ("contract_name", "crates.io:drop-neutron-contracts__drop-staker"),
            ("contract_version", "1.0.0"),
            ("msg", "InstantiateMsg { connection_id: \"connection\", port_id: \"port_id\", timeout: 10, remote_denom: \"remote_denom\", base_denom: \"base_denom\", transfer_channel_id: \"transfer_channel_id\", owner: Some(\"owner\"), allowed_senders: [\"core\"], min_ibc_transfer: Uint128(10000), min_staking_amount: Uint128(10000) }"),
            ("sender", "admin")
        ]))
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    let mut default_config = get_default_config();
    default_config.puppeteer_ica = None; // puppeteer_ica is not set at the time of instantiation
    assert_eq!(config, default_config);
    let owner = cw_ownable::get_ownership(deps.as_ref().storage)
        .unwrap()
        .owner
        .unwrap();
    assert_eq!(owner, Addr::unchecked("owner"));
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = drop_staking_base::msg::staker::InstantiateMsg {
        connection_id: "connection".to_string(),
        timeout: 10u64,
        port_id: "port_id".to_string(),
        transfer_channel_id: "transfer_channel_id".to_string(),
        remote_denom: "remote_denom".to_string(),
        base_denom: "base_denom".to_string(),
        allowed_senders: vec!["core".to_string()],
        min_ibc_transfer: Uint128::from(10000u128),
        min_staking_amount: Uint128::from(10000u128),
        owner: Some("owner".to_string()),
    };
    let _res = instantiate(deps.as_mut(), mock_env(), mock_info("admin", &[]), msg).unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("admin")).unwrap();
    let msg = ConfigOptional {
        timeout: Some(20u64),
        allowed_senders: Some(vec!["new_core".to_string()]),
        puppeteer_ica: Some("puppeteer_ica".to_string()),
        min_ibc_transfer: Some(Uint128::from(110000u128)),
        min_staking_amount: Some(Uint128::from(110000u128)),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::staker::ExecuteMsg::UpdateConfig {
            new_config: Box::new(msg),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-neutron-contracts__drop-staker-update_config"
        ).add_attributes(vec![
            ("action","update_config"),
            ("new_config", "ConfigOptional { timeout: Some(20), allowed_senders: Some([\"new_core\"]), puppeteer_ica: Some(\"puppeteer_ica\"), min_ibc_transfer: Some(Uint128(110000)), min_staking_amount: Some(Uint128(110000)) }")
        ]))
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            connection_id: "connection".to_string(),
            timeout: 20u64,
            port_id: "port_id".to_string(),
            transfer_channel_id: "transfer_channel_id".to_string(),
            remote_denom: "remote_denom".to_string(),
            base_denom: "base_denom".to_string(),
            allowed_senders: vec![Addr::unchecked("new_core")],
            puppeteer_ica: Some("puppeteer_ica".to_string()),
            min_ibc_transfer: Uint128::from(110000u128),
            min_staking_amount: Uint128::from(110000u128),
        }
    );
}

#[test]
fn test_register_ica_no_fee() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let msg = drop_staking_base::msg::staker::ExecuteMsg::RegisterICA {};

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
            reason: "missing fee in denom untrn".to_string()
        }
    )
}

#[test]
fn test_register_ica() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let msg = drop_staking_base::msg::staker::ExecuteMsg::RegisterICA {};

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &coins(1000, "untrn")),
        msg.clone(),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-staker-register-ica")
                    .add_attributes(vec![
                        ("action", "register_ica"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_STAKER")
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Custom(
                NeutronMsg::register_interchain_account(
                    "connection".to_string(),
                    "drop_STAKER".to_string(),
                    Some(coins(1000, "untrn")),
                )
            )))
    );
    // already asked for registration
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &coins(1000, "untrn")),
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
        mock_info("nobody", &coins(1000, "untrn")),
        msg.clone(),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-staker-register-ica")
                    .add_attributes(vec![
                        ("action", "register_ica"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_STAKER")
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Custom(
                NeutronMsg::register_interchain_account(
                    "connection".to_string(),
                    "drop_STAKER".to_string(),
                    Some(coins(1000, "untrn")),
                )
            )))
    );
}

#[test]
fn test_ibc_transfer() {
    let mut deps = mock_dependencies(&[cosmwasm_std::Coin {
        denom: "base_denom".to_string(),
        amount: Uint128::from(10000u64),
    }]);
    {
        CONFIG
            .save(deps.as_mut().storage, &get_default_config())
            .unwrap();
        TX_STATE
            .save(
                deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::InProgress,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("nobody", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::IBCTransfer {},
        )
        .unwrap_err();
        assert_eq!(
            res,
            crate::error::ContractError::InvalidState {
                reason: "tx_state is not idle".to_string()
            }
        );
    }
    {
        let mut invalid_contract_balance_deps = mock_dependencies(&[cosmwasm_std::Coin {
            denom: "base_denom".to_string(),
            amount: Uint128::from(0u64),
        }]);
        CONFIG
            .save(
                invalid_contract_balance_deps.as_mut().storage,
                &get_default_config(),
            )
            .unwrap();
        TX_STATE
            .save(
                invalid_contract_balance_deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::Idle,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        let res = execute(
            invalid_contract_balance_deps.as_mut(),
            mock_env(),
            mock_info("nobody", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::IBCTransfer {},
        )
        .unwrap_err();
        assert_eq!(
            res,
            crate::error::ContractError::InvalidFunds {
                reason: "amount is less than min_ibc_transfer".to_string()
            }
        );
    }
    {
        NON_STAKED_BALANCE
            .save(deps.as_mut().storage, &Uint128::from(0u64))
            .unwrap();
        CONFIG
            .save(deps.as_mut().storage, &get_default_config())
            .unwrap();
        TX_STATE
            .save(
                deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::Idle,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        ICA.set_address(
            deps.as_mut().storage,
            "ica_address".to_string(),
            "port_id".to_string(),
            "channel_id".to_string(),
        )
        .unwrap();
        deps.querier.add_custom_query_response(|_| {
            to_json_binary(&MinIbcFeeResponse {
                min_fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(100, "untrn"),
                    timeout_fee: coins(200, "untrn"),
                },
            })
            .unwrap()
        });
        let env = mock_env();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("nobody", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::IBCTransfer {},
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new()
                .add_submessage(cosmwasm_std::SubMsg {
                    id: 131072u64,
                    msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                        source_port: "port_id".to_string(),
                        source_channel: "transfer_channel_id".to_string(),
                        token: cosmwasm_std::Coin {
                            denom: "base_denom".to_string(),
                            amount: Uint128::from(10000u64),
                        },
                        sender: "cosmos2contract".to_string(),
                        receiver: "ica_address".to_string(),
                        timeout_height: neutron_sdk::sudo::msg::RequestPacketTimeoutHeight {
                            revision_height: None,
                            revision_number: None
                        },
                        timeout_timestamp: env
                            .block
                            .time
                            .plus_seconds(CONFIG.load(deps.as_mut().storage).unwrap().timeout)
                            .nanos(),
                        memo: "".to_string(),
                        fee: IbcFee {
                            recv_fee: vec![],
                            ack_fee: vec![cosmwasm_std::Coin {
                                denom: "untrn".to_string(),
                                amount: Uint128::from(100u64),
                            }],
                            timeout_fee: vec![cosmwasm_std::Coin {
                                denom: "untrn".to_string(),
                                amount: Uint128::from(200u64),
                            }]
                        }
                    }),
                    gas_limit: None,
                    reply_on: cosmwasm_std::ReplyOn::Success
                })
                .add_event(
                    cosmwasm_std::Event::new(
                        "crates.io:drop-neutron-contracts__drop-staker-ibc_transfer".to_string()
                    )
                    .add_attributes(vec![
                        cosmwasm_std::attr("action".to_string(), "ibc_transfer".to_string()),
                        cosmwasm_std::attr("connection_id".to_string(), "connection".to_string()),
                        cosmwasm_std::attr("ica_id".to_string(), "drop_STAKER".to_string()),
                        cosmwasm_std::attr("pending_amount".to_string(), "10000".to_string()),
                    ])
                )
        );
    }
}

#[test]
fn test_stake() {
    let mut deps = mock_dependencies(&[]);
    {
        let config = get_default_config();
        CONFIG.save(deps.as_mut().storage, &config).unwrap();
        let msg_items = vec![
            ("validator1".to_string(), Uint128::from(5000u64)),
            ("validator2".to_string(), Uint128::from(5000u64)),
        ];
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("unauthorized_sender", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::Stake {
                items: msg_items.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(res, crate::error::ContractError::Unauthorized {})
    }
    {
        let config = get_default_config();
        CONFIG.save(deps.as_mut().storage, &config).unwrap();
        TX_STATE
            .save(
                deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        let msg_items = vec![
            ("validator1".to_string(), Uint128::from(5000u64)),
            ("validator2".to_string(), Uint128::from(5000u64)),
        ];
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("core", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::Stake {
                items: msg_items.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            crate::error::ContractError::InvalidState {
                reason: "tx_state is not idle".to_string()
            }
        )
    }
    {
        let config = get_default_config();
        CONFIG.save(deps.as_mut().storage, &config).unwrap();
        TX_STATE
            .save(
                deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::Idle,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        NON_STAKED_BALANCE
            .save(deps.as_mut().storage, &Uint128::from(0u64))
            .unwrap();
        let msg_items = vec![
            ("validator1".to_string(), Uint128::from(5000u64)),
            ("validator2".to_string(), Uint128::from(5000u64)),
        ];
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("core", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::Stake {
                items: msg_items.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            crate::error::ContractError::InvalidFunds {
                reason: "no funds to stake".to_string()
            }
        )
    }
    {
        let config = get_default_config();
        deps.querier.add_custom_query_response(|_| {
            to_json_binary(&MinIbcFeeResponse {
                min_fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(100, "untrn"),
                    timeout_fee: coins(200, "untrn"),
                },
            })
            .unwrap()
        });
        NON_STAKED_BALANCE
            .save(deps.as_mut().storage, &Uint128::from(10000u64))
            .unwrap();
        CONFIG.save(deps.as_mut().storage, &config).unwrap();
        TX_STATE
            .save(
                deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::Idle,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        ICA.set_address(
            deps.as_mut().storage,
            "ica_address".to_string(),
            "port_id".to_string(),
            "channel_id".to_string(),
        )
        .unwrap();
        let msg_items = vec![
            ("validator1".to_string(), Uint128::from(0u64)),
            ("validator2".to_string(), Uint128::from(0u64)),
        ];
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("core", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::Stake {
                items: msg_items.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            crate::error::ContractError::InvalidFunds {
                reason: "amount is less than min_staking_amount".to_string()
            }
        )
    }
    {
        let config = get_default_config();
        deps.querier.add_custom_query_response(|_| {
            to_json_binary(&MinIbcFeeResponse {
                min_fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(100, "untrn"),
                    timeout_fee: coins(200, "untrn"),
                },
            })
            .unwrap()
        });
        NON_STAKED_BALANCE
            .save(deps.as_mut().storage, &Uint128::from(123u64))
            .unwrap();
        CONFIG.save(deps.as_mut().storage, &config).unwrap();
        TX_STATE
            .save(
                deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::Idle,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        ICA.set_address(
            deps.as_mut().storage,
            "ica_address".to_string(),
            "port_id".to_string(),
            "channel_id".to_string(),
        )
        .unwrap();
        let msg_items = vec![
            ("validator1".to_string(), Uint128::from(5000u64)),
            ("validator2".to_string(), Uint128::from(5000u64)),
        ];
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("core", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::Stake {
                items: msg_items.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            crate::error::ContractError::InvalidFunds {
                reason: "not enough funds to stake".to_string()
            }
        )
    }
    {
        let config = get_default_config();
        deps.querier.add_custom_query_response(|_| {
            to_json_binary(&MinIbcFeeResponse {
                min_fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(100, "untrn"),
                    timeout_fee: coins(200, "untrn"),
                },
            })
            .unwrap()
        });
        NON_STAKED_BALANCE
            .save(deps.as_mut().storage, &Uint128::from(10000u64))
            .unwrap();
        CONFIG.save(deps.as_mut().storage, &config).unwrap();
        TX_STATE
            .save(
                deps.as_mut().storage,
                &drop_staking_base::state::staker::TxState {
                    status: drop_staking_base::state::staker::TxStateStatus::Idle,
                    seq_id: Some(0u64),
                    transaction: Some(drop_staking_base::state::staker::Transaction::Stake {
                        amount: Uint128::from(0u64),
                    }),
                    reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
                },
            )
            .unwrap();
        ICA.set_address(
            deps.as_mut().storage,
            "ica_address".to_string(),
            "port_id".to_string(),
            "channel_id".to_string(),
        )
        .unwrap();
        let msg_items = vec![
            ("validator1".to_string(), Uint128::from(5000u64)),
            ("validator2".to_string(), Uint128::from(5000u64)),
        ];
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("core", &coins(0u128, "untrn")),
            drop_staking_base::msg::staker::ExecuteMsg::Stake {
                items: msg_items.clone(),
            },
        )
        .unwrap();
        let ica_address = ICA.get_address(deps.as_mut().storage).unwrap();
        let puppeteer_ica = config.puppeteer_ica.unwrap();
        let amount_to_stake = msg_items
            .iter()
            .fold(Uint128::zero(), |acc, (_, amount)| acc + *amount);
        assert_eq!(
            res,
            cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
                id: 65536u64,
                msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::SubmitTx {
                    connection_id: "connection".to_string(),
                    interchain_account_id: "drop_STAKER".to_string(),
                    msgs: vec![neutron_sdk::bindings::types::ProtobufAny {
                        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                        value: cosmwasm_std::Binary::from(
                            cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
                                from_address: ica_address.clone(),
                                to_address: puppeteer_ica.clone(),
                                amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                    denom: config.remote_denom.to_string(),
                                    amount: amount_to_stake.to_string()
                                }]
                            }
                            .to_bytes()
                            .unwrap()
                        )
                    },
                    neutron_sdk::bindings::types::ProtobufAny {
                        type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
                        value: cosmwasm_std::Binary::from(
                            cosmos_sdk_proto::cosmos::authz::v1beta1::MsgExec {
                                grantee: ica_address.clone(),
                                msgs: msg_items
                                    .iter()
                                    .map(|(validator, amount)| {
                                        cosmos_sdk_proto::Any {
                                            type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                                            value: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate {
                                                delegator_address: puppeteer_ica.clone(),
                                                validator_address: validator.to_string(),
                                                amount: Some(
                                                    cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                                        denom: config.remote_denom.to_string(),
                                                        amount: amount.to_string(),
                                                    },
                                                ),
                                            }
                                            .to_bytes()
                                            .unwrap()
                                        }
                                    })
                                    .collect()
                            }
                            .to_bytes()
                            .unwrap()
                        )
                    }],
                    memo: "".to_string(),
                    timeout: 10u64,
                    fee: IbcFee {
                        recv_fee: vec![],
                        ack_fee: vec![cosmwasm_std::Coin {
                            denom: "untrn".to_string(),
                            amount: Uint128::from(100u64),
                        }],
                        timeout_fee: vec![cosmwasm_std::Coin {
                            denom: "untrn".to_string(),
                            amount: Uint128::from(200u64),
                        }]
                    }
                }),
                gas_limit: None,
                reply_on: cosmwasm_std::ReplyOn::Success
            })
            .add_event(cosmwasm_std::Event::new("crates.io:drop-neutron-contracts__drop-staker-stake".to_string())
                .add_attributes(vec![
                    cosmwasm_std::attr("action".to_string(), "stake".to_string()),
                    cosmwasm_std::attr("connection_id".to_string(), "connection".to_string()),
                    cosmwasm_std::attr("ica_id".to_string(), "drop_STAKER".to_string()),
                    cosmwasm_std::attr("amount_to_stake".to_string(), "10000".to_string()),
                ])
            )
        )
    }
}
