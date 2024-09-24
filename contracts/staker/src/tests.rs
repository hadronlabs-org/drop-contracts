use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use cosmos_sdk_proto::traits::MessageExt;
use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, CosmosMsg, Event, Response, SubMsg, Uint128,
};
use cosmwasm_std::{from_json, StdError};
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
        puppeteer_address: Some("puppeteer_ica".to_string()),
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
    assert_eq!(
        TX_STATE.load(deps.as_mut().storage).unwrap(),
        drop_staking_base::state::staker::TxState::default()
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    let mut default_config = get_default_config();
    default_config.puppeteer_address = None; // puppeteer_ica is not set at the time of instantiation
    assert_eq!(config, default_config);
    assert_eq!(
        NON_STAKED_BALANCE.load(deps.as_mut().storage).unwrap(),
        Uint128::zero()
    );
    let owner = cw_ownable::get_ownership(deps.as_ref().storage)
        .unwrap()
        .owner
        .unwrap();
    assert_eq!(owner, Addr::unchecked("owner"));
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::staker::InstantiateMsg {
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
        },
    )
    .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::staker::ExecuteMsg::UpdateConfig {
            new_config: Box::new(ConfigOptional {
                timeout: None,
                allowed_senders: None,
                puppeteer_address: None,
                min_ibc_transfer: None,
                min_staking_amount: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
    );
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
    let _res = instantiate(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::staker::ExecuteMsg::UpdateConfig {
            new_config: Box::new(ConfigOptional {
                timeout: Some(20u64),
                allowed_senders: Some(vec!["new_core".to_string()]),
                puppeteer_address: Some("puppeteer_ica".to_string()),
                min_ibc_transfer: Some(Uint128::from(110000u128)),
                min_staking_amount: Some(Uint128::from(110000u128)),
            }),
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
            puppeteer_address: Some("puppeteer_ica".to_string()),
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
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[]),
        drop_staking_base::msg::staker::ExecuteMsg::RegisterICA {},
    )
    .unwrap_err();
    assert_eq!(
        res,
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
fn test_ibc_transfer_not_idle() {
    let mut deps = mock_dependencies(&[]);
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

#[test]
fn test_ibc_transfer_less_than_min_ibc_transfer() {
    let mut deps = mock_dependencies(&[cosmwasm_std::Coin {
        denom: "base_denom".to_string(),
        amount: Uint128::from(0u64),
    }]);
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
    let res = execute(
        deps.as_mut(),
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

#[test]
fn test_ibc_transfer() {
    let mut deps = mock_dependencies(&[cosmwasm_std::Coin {
        denom: "base_denom".to_string(),
        amount: Uint128::from(10000u64),
    }]);
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

#[test]
fn test_stake_unauthorized() {
    let mut deps = mock_dependencies(&[]);
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

#[test]
fn test_stake_not_idle() {
    let mut deps = mock_dependencies(&[]);
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

#[test]
fn test_stake_no_funds_to_stake() {
    let mut deps = mock_dependencies(&[]);

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

#[test]
fn test_stake_less_than_min_staking_amount() {
    let mut deps = mock_dependencies(&[]);
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

#[test]
fn test_stake_not_enough_funds_to_stake() {
    let mut deps = mock_dependencies(&[]);
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
}

#[test]
fn test_stake_puppeteer_ica_not_set() {
    let mut deps = mock_dependencies(&[]);
    let mut config = get_default_config();
    config.puppeteer_address = None;
    config.min_staking_amount = Uint128::from(0u64);
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
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &coins(0u128, "untrn")),
        drop_staking_base::msg::staker::ExecuteMsg::Stake { items: vec![] },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(StdError::generic_err("puppeteer_ica not set"))
    )
}

#[test]
fn test_stake() {
    let mut deps = mock_dependencies(&[]);
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
    let puppeteer_ica = config.puppeteer_address.unwrap();
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
                    msgs: vec![
                        neutron_sdk::bindings::types::ProtobufAny {
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
                        }
                    ],
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

#[test]
fn test_sudo_response_seq_id_does_not_match() {
    let mut deps = mock_dependencies(&[]);

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
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Response {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(1u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: None,
                destination_channel: None,
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            data: cosmwasm_std::Binary::from([0; 0]),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::InvalidState {
            reason: "seq_id does not match".to_string()
        }
    )
}

#[test]
fn test_sudo_response_invalid_tx_state() {
    let mut deps = mock_dependencies(&[]);

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
        .save(deps.as_mut().storage, &Uint128::from(10000u64))
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Response {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: None,
                destination_channel: None,
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            data: cosmwasm_std::Binary::from([0; 0]),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    )
}

#[test]
fn test_sudo_response_tx_not_found() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: Some(0u64),
                transaction: None,
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Response {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            data: cosmwasm_std::Binary::from([0; 0]),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "transaction not found".to_string()
        })
    );
}

#[test]
fn test_sudo_response_ibc_client_state_not_found() {
    let mut deps = mock_dependencies(&[]);

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(10000u64))
        .unwrap();
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
    deps.querier.add_stargate_query_response(
        "/ibc.core.channel.v1.Query/ChannelClientState",
        |_data| {
            to_json_binary(
                &drop_helpers::ibc_client_state::ChannelClientStateResponse {
                    identified_client_state: None,
                    proof: None,
                    proof_height: drop_helpers::ibc_client_state::Height {
                        revision_number: cosmwasm_std::Uint64::from(0u64),
                        revision_height: cosmwasm_std::Uint64::from(33333u64),
                    },
                },
            )
            .unwrap()
        },
    );
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Response {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            data: cosmwasm_std::Binary::from([0; 0]),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "IBC client state identified_client_state not found".to_string()
        })
    );
}

#[test]
fn test_sudo_response_ibc_client_state_latest_height_not_found() {
    let mut deps = mock_dependencies(&[]);

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(10000u64))
        .unwrap();
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
    deps.querier.add_stargate_query_response(
        "/ibc.core.channel.v1.Query/ChannelClientState",
        |_data| {
            to_json_binary(
                &drop_helpers::ibc_client_state::ChannelClientStateResponse {
                    identified_client_state: Some(
                        drop_helpers::ibc_client_state::IdentifiedClientState {
                            client_id: "07-tendermint-0".to_string(),
                            client_state: drop_helpers::ibc_client_state::ClientState {
                                chain_id: "test-1".to_string(),
                                type_url: "type_url".to_string(),
                                trust_level: drop_helpers::ibc_client_state::Fraction {
                                    numerator: cosmwasm_std::Uint64::from(1u64),
                                    denominator: cosmwasm_std::Uint64::from(3u64),
                                },
                                trusting_period: Some("1000".to_string()),
                                unbonding_period: Some("1500".to_string()),
                                max_clock_drift: Some("1000".to_string()),
                                frozen_height: None,
                                latest_height: None,
                                proof_specs: vec![],
                                upgrade_path: vec![],
                                allow_update_after_expiry: true,
                                allow_update_after_misbehaviour: true,
                            },
                        },
                    ),
                    proof: None,
                    proof_height: drop_helpers::ibc_client_state::Height {
                        revision_number: cosmwasm_std::Uint64::from(0u64),
                        revision_height: cosmwasm_std::Uint64::from(33333u64),
                    },
                },
            )
            .unwrap()
        },
    );
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Response {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            data: cosmwasm_std::Binary::from([0; 0]),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "IBC client state latest_height not found".to_string()
        })
    );
}

#[test]
fn test_sudo_response() {
    let mut deps = mock_dependencies(&[]);

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(10000u64))
        .unwrap();
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
    deps.querier.add_stargate_query_response(
        "/ibc.core.channel.v1.Query/ChannelClientState",
        |_data| {
            to_json_binary(
                &drop_helpers::ibc_client_state::ChannelClientStateResponse {
                    identified_client_state: Some(
                        drop_helpers::ibc_client_state::IdentifiedClientState {
                            client_id: "07-tendermint-0".to_string(),
                            client_state: drop_helpers::ibc_client_state::ClientState {
                                chain_id: "test-1".to_string(),
                                type_url: "type_url".to_string(),
                                trust_level: drop_helpers::ibc_client_state::Fraction {
                                    numerator: cosmwasm_std::Uint64::from(1u64),
                                    denominator: cosmwasm_std::Uint64::from(3u64),
                                },
                                trusting_period: Some("1000".to_string()),
                                unbonding_period: Some("1500".to_string()),
                                max_clock_drift: Some("1000".to_string()),
                                frozen_height: None,
                                latest_height: Some(drop_helpers::ibc_client_state::Height {
                                    revision_number: cosmwasm_std::Uint64::from(0u64),
                                    revision_height: cosmwasm_std::Uint64::from(54321u64),
                                }),
                                proof_specs: vec![],
                                upgrade_path: vec![],
                                allow_update_after_expiry: true,
                                allow_update_after_misbehaviour: true,
                            },
                        },
                    ),
                    proof: None,
                    proof_height: drop_helpers::ibc_client_state::Height {
                        revision_number: cosmwasm_std::Uint64::from(0u64),
                        revision_height: cosmwasm_std::Uint64::from(33333u64),
                    },
                },
            )
            .unwrap()
        },
    );
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Response {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            data: cosmwasm_std::Binary::from([0; 0]),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_message(cosmwasm_std::CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
                    msg: to_json_binary(
                        &drop_staking_base::msg::staker::ReceiverExecuteMsg::StakerHook(
                            drop_staking_base::msg::staker::ResponseHookMsg::Success(
                                drop_staking_base::msg::staker::ResponseHookSuccessMsg {
                                    request_id: 0u64,
                                    request: neutron_sdk::sudo::msg::RequestPacket {
                                        sequence: Some(0u64),
                                        source_port: Some("source_port".to_string()),
                                        source_channel: Some("source_channel".to_string()),
                                        destination_port: Some("destination_port".to_string()),
                                        destination_channel: Some(
                                            "destination_channel".to_string()
                                        ),
                                        data: None,
                                        timeout_height: None,
                                        timeout_timestamp: None,
                                    },
                                    transaction:
                                        drop_staking_base::state::staker::Transaction::Stake {
                                            amount: Uint128::from(0u64)
                                        },
                                    local_height: 12345u64,
                                    remote_height: 54321u64,
                                }
                            )
                        )
                    )
                    .unwrap(),
                    funds: vec![]
                }
            ))
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-neutron-contracts__drop-staker-sudo-response".to_string()
                )
                .add_attributes(vec![
                    cosmwasm_std::attr("action".to_string(), "sudo_response".to_string()),
                    cosmwasm_std::attr("request_id".to_string(), "0".to_string())
                ])
            )
    );
}

#[test]
fn test_sudo_error_invalid_tx_state() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::Idle,
                seq_id: Some(0u64),
                transaction: None,
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Error {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            details: "details".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    )
}

#[test]
fn test_sudo_error_tx_not_found() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: Some(0u64),
                transaction: None,
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Error {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            details: "details".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "transaction not found".to_string()
        })
    );
}

#[test]
fn test_sudo_error_seq_id_not_found() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: Some(0u64),
                transaction: None,
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Error {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: None,
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            details: "details".to_string(),
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

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: Some(0u64),
                transaction: Some(drop_staking_base::state::staker::Transaction::IBCTransfer {
                    amount: Uint128::from(0u64),
                }),
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(0u64))
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Error {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            details: "details".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
                    msg: to_json_binary(
                        &&drop_staking_base::msg::staker::ReceiverExecuteMsg::StakerHook(
                            drop_staking_base::msg::staker::ResponseHookMsg::Error(
                                drop_staking_base::msg::staker::ResponseHookErrorMsg {
                                    request_id: 0u64,
                                    transaction:
                                        drop_staking_base::state::staker::Transaction::IBCTransfer {
                                            amount: Uint128::from(0u64)
                                        },
                                    request: neutron_sdk::sudo::msg::RequestPacket {
                                        sequence: Some(0u64),
                                        source_port: Some("source_port".to_string()),
                                        source_channel: Some("source_channel".to_string()),
                                        destination_port: Some("destination_port".to_string()),
                                        destination_channel: Some(
                                            "destination_channel".to_string()
                                        ),
                                        data: None,
                                        timeout_height: None,
                                        timeout_timestamp: None
                                    },
                                    details: "details".to_string(),
                                }
                            )
                        )
                    )
                    .unwrap(),
                    funds: vec![]
                }
            )))
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-neutron-contracts__drop-staker-sudo-error".to_string()
                )
                .add_attributes(vec![
                    cosmwasm_std::attr("action".to_string(), "sudo_error".to_string()),
                    cosmwasm_std::attr("request_id".to_string(), "0".to_string()),
                    cosmwasm_std::attr("details".to_string(), "details".to_string())
                ])
            )
    );
}

#[test]
fn test_sudo_timeout_tx_not_found() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: Some(0u64),
                transaction: None,
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Error {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            details: "details".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "transaction not found".to_string()
        })
    );
}

#[test]
fn test_sudo_timeout_seq_not_found() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: Some(0u64),
                transaction: None,
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Error {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: None,
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
            details: "details".to_string(),
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
fn test_sudo_timeout() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: Some(0u64),
                transaction: Some(drop_staking_base::state::staker::Transaction::IBCTransfer {
                    amount: Uint128::from(0u64),
                }),
                reply_to: Some("neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string()),
            },
        )
        .unwrap();
    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(0u64))
        .unwrap();
    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::Timeout {
            request: neutron_sdk::sudo::msg::RequestPacket {
                sequence: Some(0u64),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: None,
                timeout_height: None,
                timeout_timestamp: None,
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
                    msg: to_json_binary(
                        &&drop_staking_base::msg::staker::ReceiverExecuteMsg::StakerHook(
                            drop_staking_base::msg::staker::ResponseHookMsg::Error(
                                drop_staking_base::msg::staker::ResponseHookErrorMsg {
                                    request_id: 0u64,
                                    transaction:
                                        drop_staking_base::state::staker::Transaction::IBCTransfer {
                                            amount: Uint128::from(0u64)
                                        },
                                    request: neutron_sdk::sudo::msg::RequestPacket {
                                        sequence: Some(0u64),
                                        source_port: Some("source_port".to_string()),
                                        source_channel: Some("source_channel".to_string()),
                                        destination_port: Some("destination_port".to_string()),
                                        destination_channel: Some(
                                            "destination_channel".to_string()
                                        ),
                                        data: None,
                                        timeout_height: None,
                                        timeout_timestamp: None
                                    },
                                    details: "timeout".to_string(),
                                }
                            )
                        )
                    )
                    .unwrap(),
                    funds: vec![]
                }
            )))
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-neutron-contracts__drop-staker-sudo-timeout".to_string()
                )
                .add_attributes(vec![
                    cosmwasm_std::attr("action".to_string(), "sudo_timeout".to_string()),
                    cosmwasm_std::attr("request_id".to_string(), "0".to_string())
                ])
            )
    );
}

#[test]
fn test_sudo_open_ack_invalid_version() {
    let mut deps = mock_dependencies(&[]);

    let res = crate::contract::sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::SudoMsg::OpenAck {
            port_id: "port_id_1".to_string(),
            channel_id: "channel_1".to_string(),
            counterparty_channel_id: "counterparty_channel_id_1".to_string(),
            counterparty_version: "".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::Std(StdError::generic_err("can't parse version",))
    );
}

#[test]
fn test_sudo_open_ack() {
    let mut deps = mock_dependencies(&[]);

    let res = crate::contract::sudo(deps.as_mut(), mock_env(), neutron_sdk::sudo::msg::SudoMsg::OpenAck {
            port_id: "port_id_1".to_string(),
            channel_id: "channel_1".to_string(),
            counterparty_channel_id: "counterparty_channel_id_1".to_string(),
            counterparty_version: "{\"version\": \"1\",\"controller_connection_id\": \"connection_id\",\"host_connection_id\": \"host_connection_id\",\"address\": \"ica_address\",\"encoding\": \"amino\",\"tx_type\": \"cosmos-sdk/MsgSend\"}".to_string(),
        }).unwrap();
    assert_eq!(res, cosmwasm_std::Response::new());
}

#[test]
fn test_reply_submit_tx_reply_no_result() {
    let mut deps = mock_dependencies(&[]);

    let res = crate::contract::reply(
        deps.as_mut().into_empty(),
        mock_env(),
        cosmwasm_std::Reply {
            id: drop_staking_base::state::staker::reply_msg::SUDO_PAYLOAD,
            result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        StdError::GenericErr {
            msg: "no result".to_string(),
        }
    );
}

#[test]
fn test_reply_submit_tx_reply() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: None,
                transaction: None,
                reply_to: None,
            },
        )
        .unwrap();
    let res = crate::contract::reply(
        deps.as_mut().into_empty(),
        mock_env(),
        cosmwasm_std::Reply {
            id: drop_staking_base::state::staker::reply_msg::SUDO_PAYLOAD,
            result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                events: vec![],
                data: Some(
                    to_json_binary(&neutron_sdk::bindings::msg::MsgSubmitTxResponse {
                        sequence_id: 0u64,
                        channel: "channel-0".to_string(),
                    })
                    .unwrap(),
                ),
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new("puppeteer-base-reply-tx-payload-received".to_string())
                .add_attributes(vec![
                    cosmwasm_std::attr("channel_id".to_string(), "channel-0".to_string()),
                    cosmwasm_std::attr("seq_id".to_string(), "0".to_string())
                ])
        )
    )
}

#[test]
fn test_reply_submit_ibc_transfer_no_result() {
    let mut deps = mock_dependencies(&[]);

    let res = crate::contract::reply(
        deps.as_mut().into_empty(),
        mock_env(),
        cosmwasm_std::Reply {
            id: drop_staking_base::state::staker::reply_msg::IBC_TRANSFER,
            result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                events: vec![],
                data: None,
            }),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        StdError::GenericErr {
            msg: "no result".to_string(),
        }
    );
}

#[test]
fn test_reply_submit_ibc_transfer() {
    let mut deps = mock_dependencies(&[]);

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::staker::TxState {
                status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
                seq_id: None,
                transaction: None,
                reply_to: None,
            },
        )
        .unwrap();
    let res = crate::contract::reply(
        deps.as_mut().into_empty(),
        mock_env(),
        cosmwasm_std::Reply {
            id: drop_staking_base::state::staker::reply_msg::IBC_TRANSFER,
            result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                events: vec![],
                data: Some(
                    to_json_binary(&neutron_sdk::bindings::msg::MsgSubmitTxResponse {
                        sequence_id: 0u64,
                        channel: "channel-0".to_string(),
                    })
                    .unwrap(),
                ),
            }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "puppeteer-base-reply-ibc-transfer-payload-received".to_string()
            )
            .add_attributes(vec![
                cosmwasm_std::attr("channel_id".to_string(), "channel-0".to_string()),
                cosmwasm_std::attr("seq_id".to_string(), "0".to_string())
            ])
        )
    )
}

#[test]
fn test_query_tx_state() {
    let mut deps = mock_dependencies(&[]);
    let tx_state = drop_staking_base::state::staker::TxState {
        status: drop_staking_base::state::staker::TxStateStatus::WaitingForAck,
        seq_id: None,
        transaction: None,
        reply_to: None,
    };
    TX_STATE.save(deps.as_mut().storage, &tx_state).unwrap();
    let res: drop_staking_base::state::staker::TxState = from_json(
        crate::contract::query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::staker::QueryMsg::TxState {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, tx_state);
}

#[test]
fn test_query_non_staked_balance() {
    let mut deps = mock_dependencies(&[]);
    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(10000u64))
        .unwrap();
    let res: Uint128 = from_json(
        crate::contract::query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::staker::QueryMsg::NonStakedBalance {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, Uint128::from(10000u64));
}

#[test]
fn test_query_all_balance() {
    let mut deps = mock_dependencies(&[cosmwasm_std::Coin {
        denom: "base_denom".to_string(),
        amount: Uint128::from(123u64),
    }]);
    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(10000u64))
        .unwrap();
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let res: Uint128 = from_json(
        crate::contract::query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::staker::QueryMsg::AllBalance {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, Uint128::from(10123u64));
}

#[test]
fn test_query_ica() {
    let mut deps = mock_dependencies(&[]);
    ICA.set_address(
        deps.as_mut().storage,
        "ica_address".to_string(),
        "port_id".to_string(),
        "channel_id".to_string(),
    )
    .unwrap();
    let res: drop_helpers::ica::IcaState = from_json(
        crate::contract::query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::staker::QueryMsg::Ica {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        drop_helpers::ica::IcaState::Registered {
            ica_address: "ica_address".to_string(),
            port_id: "port_id".to_string(),
            channel_id: "channel_id".to_string(),
        }
    );
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let res: drop_staking_base::state::staker::Config = from_json(
        crate::contract::query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::staker::QueryMsg::Config {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(res, get_default_config());
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
        drop_staking_base::msg::staker::ExecuteMsg::UpdateOwnership(
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
        drop_staking_base::msg::staker::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::AcceptOwnership {},
        ),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        crate::contract::query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::staker::QueryMsg::Ownership {},
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
