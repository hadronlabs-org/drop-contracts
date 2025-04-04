use crate::contract::{
    execute, get_contract_config_owner, get_contract_version, instantiate, query,
    validate_contract_metadata,
};
use cosmwasm_std::Binary;
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, DepsMut, Uint128,
};
use drop_helpers::phonebook::{
    CORE_CONTRACT, DISTRIBUTION_CONTRACT, LSM_SHARE_BOND_PROVIDER_CONTRACT,
    NATIVE_BOND_PROVIDER_CONTRACT, PUPPETEER_CONTRACT, REWARDS_MANAGER_CONTRACT,
    REWARDS_PUMP_CONTRACT, SPLITTER_CONTRACT, STRATEGY_CONTRACT, TOKEN_CONTRACT,
    VALIDATORS_SET_CONTRACT, WITHDRAWAL_MANAGER_CONTRACT, WITHDRAWAL_VOUCHER_CONTRACT,
};
use drop_helpers::{
    pause::Interval,
    testing::{mock_dependencies, mock_dependencies_with_api, WasmMockQuerier},
};
use drop_staking_base::state::factory::PreInstantiatedContracts;
use drop_staking_base::{
    msg::factory::{
        CoreParams, ExecuteMsg, FeeParams, InstantiateMsg, QueryMsg, UpdateConfigMsg,
        ValidatorSetMsg,
    },
    state::factory::{CodeIds, RemoteOpts, Timeout, STATE},
};
use drop_staking_base::{
    msg::{
        core::{ExecuteMsg as CoreExecuteMsg, InstantiateMsg as CoreInstantiateMsg},
        distribution::InstantiateMsg as DistributionInstantiateMsg,
        puppeteer::ExecuteMsg as PuppeteerExecuteMsg,
        rewards_manager::InstantiateMsg as RewardsManagerInstantiateMsg,
        splitter::InstantiateMsg as SplitterInstantiateMsg,
        strategy::InstantiateMsg as StrategyInstantiateMsg,
        token::{DenomMetadata, InstantiateMsg as TokenInstantiateMsg},
        validatorset::{
            ExecuteMsg as ValidatorSetExecuteMsg, InstantiateMsg as ValidatorsSetInstantiateMsg,
        },
        withdrawal_manager::InstantiateMsg as WithdrawalManagerInstantiateMsg,
        withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
    },
    state::{core::Pause as CorePause, splitter::Config as SplitterConfig},
};
use neutron_sdk::bindings::query::NeutronQuery;
use std::collections::HashMap;

fn set_default_factory_state(deps: DepsMut<NeutronQuery>) {
    STATE
        .save(
            deps.storage,
            TOKEN_CONTRACT,
            &Addr::unchecked("token_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            CORE_CONTRACT,
            &Addr::unchecked("core_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            PUPPETEER_CONTRACT,
            &Addr::unchecked("puppeteer_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            WITHDRAWAL_MANAGER_CONTRACT,
            &Addr::unchecked("withdrawal_manager_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            WITHDRAWAL_VOUCHER_CONTRACT,
            &Addr::unchecked("withdrawal_voucher_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            STRATEGY_CONTRACT,
            &Addr::unchecked("strategy_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            VALIDATORS_SET_CONTRACT,
            &Addr::unchecked("validators_set_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            DISTRIBUTION_CONTRACT,
            &Addr::unchecked("distribution_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            REWARDS_MANAGER_CONTRACT,
            &Addr::unchecked("rewards_manager_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            REWARDS_PUMP_CONTRACT,
            &Addr::unchecked("rewards_pump_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            SPLITTER_CONTRACT,
            &Addr::unchecked("splitter_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            LSM_SHARE_BOND_PROVIDER_CONTRACT,
            &Addr::unchecked("lsm_share_bond_provider_contract"),
        )
        .unwrap();
    STATE
        .save(
            deps.storage,
            NATIVE_BOND_PROVIDER_CONTRACT,
            &Addr::unchecked("native_bond_provider_contract"),
        )
        .unwrap();
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies_with_api(&[]);
    deps.querier
        .add_stargate_query_response("/cosmos.wasm.v1.Query/QueryCodeRequest", |data| {
            let mut y = vec![0; 32];
            y[..data.len()].copy_from_slice(data);
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&cosmwasm_std::CodeInfoResponse::new(
                    from_json(data).unwrap(),
                    "creator".to_string(),
                    cosmwasm_std::HexBinary::from(y.as_slice()),
                ))
                .unwrap(),
            )
        });

    let mocked_env = cosmwasm_std::Env {
        block: cosmwasm_std::BlockInfo {
            height: 12_345,
            time: cosmwasm_std::Timestamp::from_nanos(1_571_797_419_879_305_533),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        transaction: Some(cosmwasm_std::TransactionInfo { index: 3 }),
        contract: cosmwasm_std::ContractInfo {
            address: cosmwasm_std::Addr::unchecked("factory_contract"),
        },
    };

    let contract_admin = mocked_env.contract.address.to_string();

    setup_contract_metadata(
        &mut deps.querier,
        "native_bond_provider_address",
        "crates.io:drop-staking__drop-native-bond-provider".to_string(),
        mocked_env.contract.address.to_string(),
        Some(contract_admin.clone()),
    );
    setup_contract_metadata(
        &mut deps.querier,
        "puppeteer_address",
        "crates.io:drop-staking__drop-puppeteer".to_string(),
        mocked_env.contract.address.to_string(),
        Some(contract_admin),
    );

    let instantiate_msg = InstantiateMsg {
        code_ids: CodeIds {
            token_code_id: 1,
            core_code_id: 2,
            withdrawal_voucher_code_id: 5,
            withdrawal_manager_code_id: 6,
            strategy_code_id: 7,
            validators_set_code_id: 8,
            distribution_code_id: 9,
            rewards_manager_code_id: 10,
            splitter_code_id: 12,
        },
        pre_instantiated_contracts: PreInstantiatedContracts {
            native_bond_provider_address: cosmwasm_std::Addr::unchecked(
                "native_bond_provider_address",
            ),
            puppeteer_address: cosmwasm_std::Addr::unchecked("puppeteer_address"),
            lsm_share_bond_provider_address: None,
            unbonding_pump_address: None,
            rewards_pump_address: None,
            val_ref_address: None,
        },
        remote_opts: RemoteOpts {
            denom: "denom".to_string(),
            connection_id: "connection-0".to_string(),
            transfer_channel_id: "channel-0".to_string(),
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
        base_denom: "base_denom".to_string(),
        local_denom: "local-denom".to_string(),
        core_params: CoreParams {
            idle_min_interval: 0,
            unbonding_period: 0,
            unbonding_safe_period: 0,
            unbond_batch_switch_time: 0,
            icq_update_delay: 0,
        },
        fee_params: Some(FeeParams {
            fee: cosmwasm_std::Decimal::new(Uint128::from(0u64)),
            fee_address: "fee_address".to_string(),
        }),
    };
    let res = instantiate(
        deps.as_mut().into_empty(),
        mocked_env,
        mock_info("owner", &[]),
        instantiate_msg,
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_submessages(vec![
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 1,
                        label: "drop-staking-token".to_string(),
                        msg: to_json_binary(&TokenInstantiateMsg {
                            factory_contract: "factory_contract".to_string(),
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
                            owner: "factory_contract".to_string()
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 8,
                        label: "drop-staking-validators-set".to_string(),
                        msg: to_json_binary(&ValidatorsSetInstantiateMsg {
                            owner: "factory_contract".to_string(),
                            stats_contract: "neutron1x69dz0c0emw8m2c6kp5v6c08kgjxmu30f4a8w5"
                                .to_string()
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 9,
                        label: "drop-staking-distribution".to_string(),
                        msg: to_json_binary(&DistributionInstantiateMsg {}).unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 7,
                        label: "drop-staking-strategy".to_string(),
                        msg: to_json_binary(&StrategyInstantiateMsg {
                            owner: "factory_contract".to_string(),
                            factory_contract: "factory_contract".to_string(),
                            denom: "denom".to_string()
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 2,
                        label: "drop-staking-core".to_string(),
                        msg: to_json_binary(&CoreInstantiateMsg {
                            factory_contract: "factory_contract".to_string(),
                            base_denom: "base_denom".to_string(),
                            remote_denom: "denom".to_string(),
                            idle_min_interval: 0,
                            unbonding_period: 0,
                            unbonding_safe_period: 0,
                            unbond_batch_switch_time: 0,
                            pump_ica_address: None,
                            owner: "factory_contract".to_string(),
                            emergency_address: None,
                            icq_update_delay: 0
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 5,
                        label: "drop-staking-withdrawal-voucher".to_string(),
                        msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
                            name: "Drop Voucher".to_string(),
                            symbol: "DROPV".to_string(),
                            minter: "some_humanized_address".to_string(),
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 6,
                        label: "drop-staking-withdrawal-manager".to_string(),
                        msg: to_json_binary(&WithdrawalManagerInstantiateMsg {
                            factory_contract: "factory_contract".to_string(),
                            base_denom: "base_denom".to_string(),
                            owner: "factory_contract".to_string()
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 10,
                        label: "drop-staking-rewards-manager".to_string(),
                        msg: to_json_binary(&RewardsManagerInstantiateMsg {
                            owner: "factory_contract".to_string()
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 12,
                        label: "drop-staking-splitter".to_string(),
                        msg: to_json_binary(&SplitterInstantiateMsg {
                            config: SplitterConfig {
                                receivers: vec![
                                    (
                                        "native_bond_provider_address".to_string(),
                                        Uint128::from(10000u64)
                                    ),
                                    ("fee_address".to_string(), Uint128::from(0u64))
                                ],
                                denom: "base_denom".to_string()
                            }
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                ))
            ])
            .add_event(
                cosmwasm_std::Event::new("crates.io:drop-staking__drop-factory-instantiate")
                    .add_attributes(vec![
                        attr("base_denom", "base_denom"),
                        attr("salt", "salt"),
                        attr("owner", "owner"),
                        attr("subdenom", "subdenom"),
                    ])
            )
    );
    cw_ownable::assert_owner(
        deps.as_mut().storage,
        &cosmwasm_std::Addr::unchecked("owner".to_string()),
    )
    .unwrap();
}

#[test]
fn test_update_config_core_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let new_core_config = drop_staking_base::state::core::ConfigOptional {
        factory_contract: None,
        base_denom: None,
        remote_denom: None,
        idle_min_interval: None,
        unbonding_period: None,
        unbonding_safe_period: None,
        unbond_batch_switch_time: None,
        pump_ica_address: None,
        rewards_receiver: None,
        emergency_address: None,
    };
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        ExecuteMsg::UpdateConfig(Box::new(UpdateConfigMsg::Core(Box::new(
            new_core_config.clone(),
        )))),
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::factory::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn test_update_config_core() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    set_default_factory_state(deps.as_mut());
    let new_core_config = drop_staking_base::state::core::ConfigOptional {
        factory_contract: Some("factory_contract1".to_string()),
        base_denom: Some("base_denom1".to_string()),
        remote_denom: Some("remote_denom1".to_string()),
        idle_min_interval: Some(1u64),
        unbonding_period: Some(1u64),
        unbonding_safe_period: Some(1u64),
        unbond_batch_switch_time: Some(1u64),
        pump_ica_address: Some("pump_ica_address1".to_string()),
        rewards_receiver: Some("rewards_receiver1".to_string()),
        emergency_address: Some("emergency_address1".to_string()),
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

#[test]
fn test_update_config_validators_set_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let new_validator_set_config = drop_staking_base::state::validatorset::ConfigOptional {
        stats_contract: Some("validator_stats_contract".to_string()),
        provider_proposals_contract: Some("provider_proposals_contract1".to_string()),
        val_ref_contract: Some("val_ref_contract1".to_string()),
    };
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        ExecuteMsg::UpdateConfig(Box::new(UpdateConfigMsg::ValidatorsSet(
            new_validator_set_config.clone(),
        ))),
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::factory::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn test_update_config_validators_set() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    set_default_factory_state(deps.as_mut());

    let new_validator_set_config = drop_staking_base::state::validatorset::ConfigOptional {
        stats_contract: Some("validator_stats_contract".to_string()),
        provider_proposals_contract: Some("provider_proposals_contract1".to_string()),
        val_ref_contract: Some("val_ref_contract1".to_string()),
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

#[test]
fn test_proxy_validators_set_update_validators_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    set_default_factory_state(deps.as_mut());
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        ExecuteMsg::Proxy(drop_staking_base::msg::factory::ProxyMsg::ValidatorSet(
            ValidatorSetMsg::UpdateValidators {
                validators: vec![
                    drop_staking_base::msg::validatorset::ValidatorData {
                        valoper_address: "valoper_address1".to_string(),
                        weight: 10u64,
                        on_top: Some(Uint128::zero()),
                    },
                    drop_staking_base::msg::validatorset::ValidatorData {
                        valoper_address: "valoper_address2".to_string(),
                        weight: 10u64,
                        on_top: Some(Uint128::zero()),
                    },
                ],
            },
        )),
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::factory::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn test_proxy_validators_set_update_validators() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    set_default_factory_state(deps.as_mut());

    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::Proxy(drop_staking_base::msg::factory::ProxyMsg::ValidatorSet(
            ValidatorSetMsg::UpdateValidators {
                validators: vec![
                    drop_staking_base::msg::validatorset::ValidatorData {
                        valoper_address: "valoper_address1".to_string(),
                        weight: 10u64,
                        on_top: Some(Uint128::zero()),
                    },
                    drop_staking_base::msg::validatorset::ValidatorData {
                        valoper_address: "valoper_address2".to_string(),
                        weight: 10u64,
                        on_top: Some(Uint128::zero()),
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
                                    on_top: Some(Uint128::zero()),
                                },
                                drop_staking_base::msg::validatorset::ValidatorData {
                                    valoper_address: "valoper_address2".to_string(),
                                    weight: 10u64,
                                    on_top: Some(Uint128::zero()),
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

#[test]
fn test_admin_execute_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        ExecuteMsg::AdminExecute {
            msgs: vec![
                cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "core_contract".to_string(),
                    msg: to_json_binary(&CoreExecuteMsg::SetPause(CorePause {
                        tick: Interval {
                            from: 1000,
                            to: 1000000,
                        },
                        bond: Interval {
                            from: 1000,
                            to: 1000000,
                        },
                        unbond: Interval { from: 0, to: 0 },
                    }))
                    .unwrap(),
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
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::factory::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
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
                    msg: to_json_binary(&CoreExecuteMsg::SetPause(CorePause {
                        tick: Interval {
                            from: 1000,
                            to: 1000000,
                        },
                        bond: Interval {
                            from: 1000,
                            to: 1000000,
                        },
                        unbond: Interval { from: 0, to: 0 },
                    }))
                    .unwrap(),
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
                    msg: to_json_binary(&CoreExecuteMsg::SetPause(CorePause {
                        tick: Interval {
                            from: 1000,
                            to: 1000000,
                        },
                        bond: Interval {
                            from: 1000,
                            to: 1000000,
                        },
                        unbond: Interval { from: 0, to: 0 },
                    }))
                    .unwrap(),
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
fn test_query_state() {
    let mut deps = mock_dependencies(&[]);
    set_default_factory_state(deps.as_mut());
    let query_res: HashMap<String, String> =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()).unwrap();
    assert_eq!(
        query_res,
        HashMap::from([
            ("core_contract".to_string(), "core_contract".to_string()),
            ("token_contract".to_string(), "token_contract".to_string()),
            (
                "puppeteer_contract".to_string(),
                "puppeteer_contract".to_string()
            ),
            (
                "withdrawal_voucher_contract".to_string(),
                "withdrawal_voucher_contract".to_string()
            ),
            (
                "withdrawal_manager_contract".to_string(),
                "withdrawal_manager_contract".to_string()
            ),
            (
                "strategy_contract".to_string(),
                "strategy_contract".to_string()
            ),
            (
                "validators_set_contract".to_string(),
                "validators_set_contract".to_string()
            ),
            (
                "distribution_contract".to_string(),
                "distribution_contract".to_string()
            ),
            (
                "rewards_manager_contract".to_string(),
                "rewards_manager_contract".to_string()
            ),
            (
                "rewards_pump_contract".to_string(),
                "rewards_pump_contract".to_string()
            ),
            (
                "splitter_contract".to_string(),
                "splitter_contract".to_string()
            ),
            (
                "lsm_share_bond_provider_contract".to_string(),
                "lsm_share_bond_provider_contract".to_string()
            ),
            (
                "native_bond_provider_contract".to_string(),
                "native_bond_provider_contract".to_string()
            )
        ])
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
            drop_staking_base::msg::factory::QueryMsg::Ownership {},
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
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: "new_owner".to_string(),
            expiry: Some(cw_ownable::Expiration::Never {}),
        }),
    )
    .unwrap();
    execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("new_owner", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership {}),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::factory::QueryMsg::Ownership {},
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
        drop_staking_base::msg::factory::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::factory::ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}

#[test]
fn test_get_contract_version() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";

    let expected_data = cw2::ContractVersion {
        contract: contract_addr.to_string(),
        version: "1.0.0".to_string(),
    };

    let query_response = to_json_binary(&expected_data).unwrap();
    deps.querier
        .add_wasm_query_response(contract_addr, move |key| {
            let key_string = std::str::from_utf8(key.as_slice()).unwrap();
            assert_eq!(key_string, "contract_info");
            cosmwasm_std::ContractResult::Ok(query_response.clone())
        });

    let response = get_contract_version(
        deps.as_ref().into_empty(),
        &cosmwasm_std::Addr::unchecked(contract_addr),
    )
    .unwrap();
    assert_eq!(response, expected_data);
}

#[test]
fn test_get_contract_version_failed() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";

    deps.querier.add_wasm_query_response(contract_addr, |_| {
        cosmwasm_std::ContractResult::Ok(Binary::from(vec![]))
    });

    let error = get_contract_version(
        deps.as_ref().into_empty(),
        &cosmwasm_std::Addr::unchecked(contract_addr),
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::factory::ContractError::AbsentContractVersion {}
    );
}

#[test]
fn test_get_contract_config_owner() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";

    deps.querier.add_wasm_query_response(contract_addr, |_| {
        cosmwasm_std::ContractResult::Ok(
            to_json_binary(&cw_ownable::Ownership::<Addr> {
                owner: Some(Addr::unchecked("owner")),
                pending_owner: None,
                pending_expiry: None,
            })
            .unwrap(),
        )
    });

    let response =
        get_contract_config_owner(deps.as_ref().into_empty(), &Addr::unchecked(contract_addr))
            .unwrap();
    assert_eq!(response, "owner");
}

#[test]
fn test_validate_contract_metadata() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";
    let mocked_env = mock_env();

    let contract_admin = mocked_env.contract.address.to_string();

    setup_contract_metadata(
        &mut deps.querier,
        contract_addr,
        "contract_name".to_string(),
        mocked_env.contract.address.to_string(),
        Some(contract_admin),
    );

    validate_contract_metadata(
        deps.as_ref().into_empty(),
        &mocked_env,
        &Addr::unchecked(contract_addr),
        &["contract_name"],
    )
    .unwrap();
}

#[test]
fn test_validate_contract_metadata_two_names() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";
    let mocked_env = mock_env();

    let contract_admin = mocked_env.contract.address.to_string();

    setup_contract_metadata(
        &mut deps.querier,
        contract_addr,
        "contract_name".to_string(),
        mocked_env.contract.address.to_string(),
        Some(contract_admin),
    );

    validate_contract_metadata(
        deps.as_ref().into_empty(),
        &mocked_env,
        &Addr::unchecked(contract_addr),
        &["another_valid_name", "contract_name"],
    )
    .unwrap();
}

#[test]
fn test_validate_contract_metadata_wrong_contract_name() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";
    let mocked_env = mock_env();

    // let contract_admin = mocked_env.contract.address.to_string();

    setup_contract_metadata(
        &mut deps.querier,
        contract_addr,
        "wrong_name".to_string(),
        mocked_env.contract.address.to_string(),
        None,
    );

    let error = validate_contract_metadata(
        deps.as_ref().into_empty(),
        &mocked_env,
        &Addr::unchecked(contract_addr),
        &["contract_name"],
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::factory::ContractError::InvalidContractName {
            expected: "contract_name".to_string(),
            actual: "wrong_name".to_string()
        }
    );
}

#[test]
fn test_validate_contract_metadata_wrong_owner() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";
    let mocked_env = mock_env();

    setup_contract_metadata(
        &mut deps.querier,
        contract_addr,
        "contract_name".to_string(),
        "wrong_owner_address".to_string(),
        None,
    );

    let error = validate_contract_metadata(
        deps.as_ref().into_empty(),
        &mocked_env,
        &Addr::unchecked(contract_addr),
        &["contract_name"],
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::factory::ContractError::InvalidContractOwner {
            expected: "cosmos2contract".to_string(),
            actual: "wrong_owner_address".to_string()
        }
    );
}

#[test]
fn test_validate_contract_metadata_wrong_admin() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";
    let mocked_env = mock_env();

    setup_contract_metadata(
        &mut deps.querier,
        contract_addr,
        "contract_name".to_string(),
        mocked_env.contract.address.to_string(),
        Some("wrong_contract_admin".to_string()),
    );

    let error = validate_contract_metadata(
        deps.as_ref().into_empty(),
        &mocked_env,
        &Addr::unchecked(contract_addr),
        &["contract_name"],
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::factory::ContractError::InvalidContractAdmin {
            expected: "cosmos2contract".to_string(),
            actual: "wrong_contract_admin".to_string()
        }
    );
}

#[test]
fn test_validate_contract_metadata_empty_admin() {
    let mut deps = mock_dependencies(&[]);

    let contract_addr = "cosmos2contract";
    let mocked_env = mock_env();

    setup_contract_metadata(
        &mut deps.querier,
        contract_addr,
        "contract_name".to_string(),
        mocked_env.contract.address.to_string(),
        None,
    );

    let error = validate_contract_metadata(
        deps.as_ref().into_empty(),
        &mocked_env,
        &Addr::unchecked(contract_addr),
        &["contract_name"],
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::factory::ContractError::InvalidContractAdmin {
            expected: "cosmos2contract".to_string(),
            actual: "None".to_string()
        }
    );
}

fn setup_contract_metadata(
    querier: &mut WasmMockQuerier,
    contract_addr: &str,
    contract_name: String,
    factory_address: String,
    contract_admin: Option<String>,
) {
    querier.add_wasm_query_response(contract_addr, move |_| {
        cosmwasm_std::ContractResult::Ok(
            to_json_binary(&cw2::ContractVersion {
                contract: contract_name.clone(),
                version: "1.0.0".to_string(),
            })
            .unwrap(),
        )
    });

    querier.add_wasm_query_response(contract_addr, move |_| {
        cosmwasm_std::ContractResult::Ok(
            to_json_binary(&cw_ownable::Ownership::<Addr> {
                owner: Some(Addr::unchecked(factory_address.clone())),
                pending_owner: None,
                pending_expiry: None,
            })
            .unwrap(),
        )
    });

    querier.add_wasm_query_response(contract_addr, move |_| {
        let mut response = cosmwasm_std::ContractInfoResponse::default();
        response.admin = contract_admin.clone();

        cosmwasm_std::ContractResult::Ok(to_json_binary(&response).unwrap())
    });
}
