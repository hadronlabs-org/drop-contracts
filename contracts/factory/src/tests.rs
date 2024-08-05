use crate::{
    contract::{execute, instantiate},
    msg::{
        CoreMsg, CoreParams, ExecuteMsg, FeeParams, InstantiateMsg, ProxyMsg, StakerParams,
        UpdateConfigMsg, ValidatorSetMsg,
    },
    state::{CodeIds, RemoteOpts, State, Timeout, STATE},
};
use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info},
    to_json_binary, BankMsg, Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::msg::{
    core::ExecuteMsg as CoreExecuteMsg, puppeteer::ExecuteMsg as PuppeteerExecuteMsg,
    rewards_manager::ExecuteMsg as RewardsManagerExecuteMsg, token::DenomMetadata,
    validatorset::ExecuteMsg as ValidatorSetExecuteMsg,
    withdrawal_manager::ExecuteMsg as WithdrawalManagerExecuteMsg,
};

fn get_default_factory_state() -> State {
    State {
        token_contract: "token_contract".to_string(),
        core_contract: "core_contract".to_string(),
        puppeteer_contract: "puppeteer_contract".to_string(),
        staker_contract: "staker_contract".to_string(),
        withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
        withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
        strategy_contract: "strategy_contract".to_string(),
        validators_set_contract: "validators_set_contract".to_string(),
        distribution_contract: "distribution_contract".to_string(),
        rewards_manager_contract: "rewards_manager_contract".to_string(),
        rewards_pump_contract: "rewards_pump_contract".to_string(),
        splitter_contract: "splitter_contract".to_string(),
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    STATE
        .save(deps_mut.storage, &get_default_factory_state())
        .unwrap();
    deps.querier.add_stargate_query_response(
        "/cosmos.wasm.v1.Query/QueryCodeRequest",
        |_| -> cosmwasm_std::Binary {
            to_json_binary(&cosmwasm_std::CodeInfoResponse::new(
                100u64,
                "cosmwasm100000000000000000000000000000000000000".to_string(),
                cosmwasm_std::HexBinary::from(&[0; 32]),
            ))
            .unwrap()
        },
    );
    let instantiate_msg = InstantiateMsg {
        code_ids: CodeIds {
            token_code_id: 0,
            core_code_id: 0,
            puppeteer_code_id: 0,
            staker_code_id: 0,
            withdrawal_voucher_code_id: 0,
            withdrawal_manager_code_id: 0,
            strategy_code_id: 0,
            validators_set_code_id: 0,
            distribution_code_id: 0,
            rewards_manager_code_id: 0,
            rewards_pump_code_id: 0,
            splitter_code_id: 0,
        },
        remote_opts: RemoteOpts {
            denom: "denom".to_string(),
            update_period: 0,
            connection_id: "connection-0".to_string(),
            port_id: "transfer".to_string(),
            transfer_channel_id: "channel-0".to_string(),
            reverse_transfer_channel_id: "channel-0".to_string(),
            timeout: Timeout {
                local: 0,
                remote: 0,
            },
        },
        salt: "salt".to_string(),
        subdenom: "subdenom".to_string(),
        token_metadata: DenomMetadata {
            exponent: 6,
            display: "drop".to_string(),
            name: "Drop Token".to_string(),
            description: "Drop Token used for testing".to_string(),
            symbol: "DROP".to_string(),
            uri: None,
            uri_hash: None,
        },
        sdk_version: "sdk-version".to_string(),
        base_denom: "base_denom".to_string(),
        local_denom: "local-denom".to_string(),
        core_params: CoreParams {
            idle_min_interval: 0,
            unbonding_period: 0,
            unbonding_safe_period: 0,
            unbond_batch_switch_time: 0,
            lsm_min_bond_amount: Uint128::from(0u64),
            lsm_redeem_threshold: 0,
            lsm_redeem_max_interval: 0,
            bond_limit: Some(Uint128::from(0u64)),
            min_stake_amount: Uint128::from(0u64),
            icq_update_delay: 0,
        },
        staker_params: StakerParams {
            min_stake_amount: Uint128::from(0u64),
            min_ibc_transfer: Uint128::from(0u64),
        },
        fee_params: Some(FeeParams {
            fee: cosmwasm_std::Decimal::new(Uint128::from(0u64)),
            fee_address: "fee_address".to_string(),
        }),
    };
    let res = instantiate(
        deps.as_mut().into_empty(),
        cosmwasm_std::Env {
            block: cosmwasm_std::BlockInfo {
                height: 12_345,
                time: cosmwasm_std::Timestamp::from_nanos(1_571_797_419_879_305_533),
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            transaction: Some(cosmwasm_std::TransactionInfo { index: 3 }),
            contract: cosmwasm_std::ContractInfo {
                address: cosmwasm_std::Addr::unchecked("core_contract"),
            },
        },
        mock_info("owner", &[]),
        instantiate_msg,
    )
    .unwrap();
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    STATE
        .save(deps_mut.storage, &get_default_factory_state())
        .unwrap();
    {
        let new_core_config = drop_staking_base::state::core::ConfigOptional {
            token_contract: Some("token_contract1".to_string()),
            puppeteer_contract: Some("puppeteer_contract1".to_string()),
            strategy_contract: Some("strategy_contract1".to_string()),
            staker_contract: Some("staker_contract1".to_string()),
            withdrawal_voucher_contract: Some("withdrawal_voucher_contract1".to_string()),
            withdrawal_manager_contract: Some("withdrawal_manager_contract1".to_string()),
            validators_set_contract: Some("validators_set_contract1".to_string()),
            base_denom: Some("base_denom1".to_string()),
            remote_denom: Some("remote_denom1".to_string()),
            idle_min_interval: Some(1u64),
            unbonding_period: Some(1u64),
            unbonding_safe_period: Some(1u64),
            unbond_batch_switch_time: Some(1u64),
            pump_ica_address: Some("pump_ica_address1".to_string()),
            transfer_channel_id: Some("channel-1".to_string()),
            lsm_min_bond_amount: Some(Uint128::from(1u64)),
            lsm_redeem_threshold: Some(1u64),
            lsm_redeem_maximum_interval: Some(1u64),
            bond_limit: Some(Uint128::from(1u64)),
            rewards_receiver: Some("rewards_receiver1".to_string()),
            emergency_address: Some("emergency_address1".to_string()),
            min_stake_amount: Some(Uint128::from(1u64)),
        };
        let res = execute(
            deps.as_mut().into_empty(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig(Box::new(UpdateConfigMsg::Core(Box::new(
                new_core_config.clone(),
            )))),
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new()
                .add_event(
                    cosmwasm_std::Event::new(
                        "crates.io:drop-staking__drop-factory-execute-update-config"
                    )
                    .add_attributes(vec![attr("action", "update-config")])
                )
                .add_submessages(vec![cosmwasm_std::SubMsg::new(
                    cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "core_contract".to_string(),
                        msg: to_json_binary(&CoreExecuteMsg::UpdateConfig {
                            new_config: Box::new(new_core_config.clone())
                        })
                        .unwrap(),
                        funds: vec![]
                    })
                )])
        );
    }
    {
        let new_validator_set_config = drop_staking_base::state::validatorset::ConfigOptional {
            stats_contract: Some("validator_stats_contract".to_string()),
            provider_proposals_contract: Some("provider_proposals_contract1".to_string()),
        };
        let res = execute(
            deps.as_mut().into_empty(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig(Box::new(UpdateConfigMsg::ValidatorsSet(
                new_validator_set_config.clone(),
            ))),
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new()
                .add_event(
                    cosmwasm_std::Event::new(
                        "crates.io:drop-staking__drop-factory-execute-update-config"
                    )
                    .add_attributes(vec![attr("action", "update-config")])
                )
                .add_submessages(vec![cosmwasm_std::SubMsg::new(
                    cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "validators_set_contract".to_string(),
                        msg: to_json_binary(&ValidatorSetExecuteMsg::UpdateConfig {
                            new_config: new_validator_set_config.clone()
                        })
                        .unwrap(),
                        funds: vec![]
                    })
                )])
        );
    }
}

#[test]
fn test_proxy() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    STATE
        .save(deps_mut.storage, &get_default_factory_state())
        .unwrap();
    {
        let res = execute(
            deps.as_mut().into_empty(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Proxy(crate::msg::ProxyMsg::ValidatorSet(
                ValidatorSetMsg::UpdateValidators {
                    validators: vec![
                        drop_staking_base::msg::validatorset::ValidatorData {
                            valoper_address: "valoper_address1".to_string(),
                            weight: 10u64,
                        },
                        drop_staking_base::msg::validatorset::ValidatorData {
                            valoper_address: "valoper_address2".to_string(),
                            weight: 10u64,
                        },
                    ],
                },
            )),
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new()
                .add_submessages(vec![
                    cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                        cosmwasm_std::WasmMsg::Execute {
                            contract_addr: "validators_set_contract".to_string(),
                            msg: to_json_binary(&ValidatorSetExecuteMsg::UpdateValidators {
                                validators: vec![
                                    drop_staking_base::msg::validatorset::ValidatorData {
                                        valoper_address: "valoper_address1".to_string(),
                                        weight: 10u64,
                                    },
                                    drop_staking_base::msg::validatorset::ValidatorData {
                                        valoper_address: "valoper_address2".to_string(),
                                        weight: 10u64,
                                    },
                                ],
                            })
                            .unwrap(),
                            funds: vec![],
                        }
                    )),
                    cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                        cosmwasm_std::WasmMsg::Execute {
                            contract_addr: "puppeteer_contract".to_string(),
                            msg: to_json_binary(
                                &PuppeteerExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
                                    validators: vec![
                                        "valoper_address1".to_string(),
                                        "valoper_address2".to_string()
                                    ]
                                }
                            )
                            .unwrap(),
                            funds: vec![]
                        }
                    ))
                ])
                .add_event(
                    cosmwasm_std::Event::new(
                        "crates.io:drop-staking__drop-factory-execute-proxy-call".to_string()
                    )
                    .add_attribute("action".to_string(), "proxy-call".to_string())
                )
        )
    }
    {
        let res = execute(
            deps.as_mut().into_empty(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Proxy(ProxyMsg::Core(CoreMsg::Pause {})),
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new()
                .add_submessage(cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "core_contract".to_string(),
                        msg: to_json_binary(&CoreExecuteMsg::Pause {}).unwrap(),
                        funds: vec![]
                    }
                )))
                .add_event(
                    cosmwasm_std::Event::new(
                        "crates.io:drop-staking__drop-factory-execute-proxy-call".to_string()
                    )
                    .add_attribute("action".to_string(), "proxy-call".to_string())
                )
        )
    }
    {
        let res = execute(
            deps.as_mut().into_empty(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Proxy(ProxyMsg::Core(CoreMsg::Unpause {})),
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new()
                .add_submessage(cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "core_contract".to_string(),
                        msg: to_json_binary(&CoreExecuteMsg::Unpause {}).unwrap(),
                        funds: vec![]
                    }
                )))
                .add_event(
                    cosmwasm_std::Event::new(
                        "crates.io:drop-staking__drop-factory-execute-proxy-call".to_string()
                    )
                    .add_attribute("action".to_string(), "proxy-call".to_string())
                )
        )
    }
}

#[test]
fn test_admin_execute() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::AdminExecute {
            msgs: vec![
                cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "core_contract".to_string(),
                    msg: to_json_binary(&CoreExecuteMsg::Pause {}).unwrap(),
                    funds: vec![],
                }),
                cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
                    to_address: "somebody".to_string(),
                    amount: vec![
                        cosmwasm_std::Coin {
                            denom: "denom1".to_string(),
                            amount: Uint128::from(10u64),
                        },
                        cosmwasm_std::Coin {
                            denom: "denom2".to_string(),
                            amount: Uint128::from(10u64),
                        },
                    ],
                }),
            ],
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "core_contract".to_string(),
                    msg: to_json_binary(&CoreExecuteMsg::Pause {}).unwrap(),
                    funds: vec![]
                }
            )))
            .add_submessage(cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Bank(
                cosmwasm_std::BankMsg::Send {
                    to_address: "somebody".to_string(),
                    amount: vec![
                        cosmwasm_std::Coin {
                            denom: "denom1".to_string(),
                            amount: Uint128::from(10u64),
                        },
                        cosmwasm_std::Coin {
                            denom: "denom2".to_string(),
                            amount: Uint128::from(10u64),
                        }
                    ]
                }
            )))
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-staking__drop-factory-execute-admin".to_string()
                )
                .add_attribute("action".to_string(), "admin-execute".to_string())
            )
    )
}

#[test]
fn test_pause() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    STATE
        .save(deps.as_mut().storage, &get_default_factory_state())
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::Pause {},
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_submessages(vec![
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "core_contract".to_string(),
                        msg: to_json_binary(&CoreExecuteMsg::Pause {}).unwrap(),
                        funds: vec![]
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "withdrawal_manager_contract".to_string(),
                        msg: to_json_binary(&WithdrawalManagerExecuteMsg::Pause {}).unwrap(),
                        funds: vec![]
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "rewards_manager_contract".to_string(),
                        msg: to_json_binary(&RewardsManagerExecuteMsg::Pause {}).unwrap(),
                        funds: vec![]
                    }
                ))
            ])
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-staking__drop-factory-execute-pause".to_string()
                )
                .add_attributes(vec![cosmwasm_std::attr(
                    "action".to_string(),
                    "pause".to_string()
                )])
            )
    )
}

#[test]
fn test_unpause() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    STATE
        .save(deps.as_mut().storage, &get_default_factory_state())
        .unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::Unpause {},
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_submessages(vec![
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "core_contract".to_string(),
                        msg: to_json_binary(&CoreExecuteMsg::Unpause {}).unwrap(),
                        funds: vec![]
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "rewards_manager_contract".to_string(),
                        msg: to_json_binary(&RewardsManagerExecuteMsg::Unpause {}).unwrap(),
                        funds: vec![]
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Execute {
                        contract_addr: "withdrawal_manager_contract".to_string(),
                        msg: to_json_binary(&WithdrawalManagerExecuteMsg::Unpause {}).unwrap(),
                        funds: vec![]
                    }
                ))
            ])
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-staking__drop-factory-execute-unpause".to_string()
                )
                .add_attributes(vec![cosmwasm_std::attr(
                    "action".to_string(),
                    "unpause".to_string()
                )])
            )
    )
}
