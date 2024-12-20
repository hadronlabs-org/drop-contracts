use crate::contract::{execute, instantiate, query};
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, DepsMut, Uint128,
};
use drop_helpers::{
    phonebook::{
        CORE_CONTRACT, DISTRIBUTION_CONTRACT, LSM_SHARE_BOND_PROVIDER_CONTRACT,
        NATIVE_BOND_PROVIDER_CONTRACT, PUPPETEER_CONTRACT, REWARDS_MANAGER_CONTRACT,
        REWARDS_PUMP_CONTRACT, SPLITTER_CONTRACT, STRATEGY_CONTRACT, TOKEN_CONTRACT,
        VALIDATORS_SET_CONTRACT, WITHDRAWAL_MANAGER_CONTRACT, WITHDRAWAL_VOUCHER_CONTRACT,
    },
    testing::{mock_dependencies, mock_dependencies_with_api},
};
use drop_staking_base::{
    msg::factory::{
        CoreParams, ExecuteMsg, FeeParams, InstantiateMsg, LsmShareBondParams, NativeBondParams,
        QueryMsg, UpdateConfigMsg, ValidatorSetMsg,
    },
    state::factory::{CodeIds, RemoteOpts, Timeout, STATE},
};
use drop_staking_base::{
    msg::{
        core::{ExecuteMsg as CoreExecuteMsg, InstantiateMsg as CoreInstantiateMsg},
        distribution::InstantiateMsg as DistributionInstantiateMsg,
        lsm_share_bond_provider::InstantiateMsg as LsmShareBondProviderInstantiateMsg,
        native_bond_provider::InstantiateMsg as NativeBondProviderInstantiateMsg,
        pump::InstantiateMsg as RewardsPumpInstantiateMsg,
        puppeteer::{ExecuteMsg as PuppeteerExecuteMsg, InstantiateMsg as PuppeteerInstantiateMsg},
        rewards_manager::{
            ExecuteMsg as RewardsManagerExecuteMsg, InstantiateMsg as RewardsManagerInstantiateMsg,
        },
        splitter::InstantiateMsg as SplitterInstantiateMsg,
        strategy::InstantiateMsg as StrategyInstantiateMsg,
        token::{DenomMetadata, InstantiateMsg as TokenInstantiateMsg},
        validatorset::{
            ExecuteMsg as ValidatorSetExecuteMsg, InstantiateMsg as ValidatorsSetInstantiateMsg,
        },
        withdrawal_manager::{
            ExecuteMsg as WithdrawalManagerExecuteMsg,
            InstantiateMsg as WithdrawalManagerInstantiateMsg,
        },
        withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
    },
    state::{core::Pause as CorePause, pump::PumpTimeout, splitter::Config as SplitterConfig},
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
    deps.querier.add_stargate_query_response(
        "/cosmos.wasm.v1.Query/QueryCodeRequest",
        |data| -> cosmwasm_std::Binary {
            let mut y = vec![0; 32];
            y[..data.len()].copy_from_slice(data);
            to_json_binary(&cosmwasm_std::CodeInfoResponse::new(
                from_json(data).unwrap(),
                "creator".to_string(),
                cosmwasm_std::HexBinary::from(y.as_slice()),
            ))
            .unwrap()
        },
    );
    let instantiate_msg = InstantiateMsg {
        code_ids: CodeIds {
            token_code_id: 1,
            core_code_id: 2,
            puppeteer_code_id: 3,
            withdrawal_voucher_code_id: 5,
            withdrawal_manager_code_id: 6,
            strategy_code_id: 7,
            validators_set_code_id: 8,
            distribution_code_id: 9,
            rewards_manager_code_id: 10,
            rewards_pump_code_id: 11,
            splitter_code_id: 12,
            lsm_share_bond_provider_code_id: 13,
            native_bond_provider_code_id: 14,
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
            icq_update_delay: 0,
        },
        native_bond_params: NativeBondParams {
            min_stake_amount: Uint128::from(0u64),
            min_ibc_transfer: Uint128::from(0u64),
        },
        fee_params: Some(FeeParams {
            fee: cosmwasm_std::Decimal::new(Uint128::from(0u64)),
            fee_address: "fee_address".to_string(),
        }),
        lsm_share_bond_params: LsmShareBondParams {
            lsm_redeem_threshold: 0,
            lsm_min_bond_amount: Uint128::from(0u64),
            lsm_redeem_max_interval: 0,
        },
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
                address: cosmwasm_std::Addr::unchecked("factory_contract"),
            },
        },
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
                        label: "validators set".to_string(),
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
                        label: "distribution".to_string(),
                        msg: to_json_binary(&DistributionInstantiateMsg {}).unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 3,
                        label: "drop-staking-puppeteer".to_string(),
                        msg: to_json_binary(&PuppeteerInstantiateMsg {
                            connection_id: "connection-0".to_string(),
                            port_id: "transfer".to_string(),
                            update_period: 0,
                            remote_denom: "denom".to_string(),
                            owner: Some("factory_contract".to_string()),
                            allowed_senders: vec![
                                "some_humanized_address".to_string(),
                                "some_humanized_address".to_string(),
                                "some_humanized_address".to_string(),
                                "factory_contract".to_string()
                            ],
                            transfer_channel_id: "channel-0".to_string(),
                            sdk_version: "sdk-version".to_string(),
                            timeout: 0,
                            delegations_queries_chunk_size: None,
                            factory_contract: "factory_contract".to_string(),
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 7,
                        label: "strategy".to_string(),
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
                            transfer_channel_id: "channel-0".to_string(),
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
                                        "some_humanized_address".to_string(),
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
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 11,
                        label: "drop-staking-rewards-pump".to_string(),
                        msg: to_json_binary(&RewardsPumpInstantiateMsg {
                            dest_address: Some("some_humanized_address".to_string()),
                            dest_channel: Some("channel-0".to_string()),
                            dest_port: Some("transfer".to_string()),
                            connection_id: "connection-0".to_string(),
                            refundee: None,
                            timeout: PumpTimeout {
                                local: Some(0),
                                remote: 0
                            },
                            local_denom: "local-denom".to_string(),
                            owner: Some("factory_contract".to_string())
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes())
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 13,
                        label: "drop-staking-lsm-share-bond-provider".to_string(),
                        msg: to_json_binary(&LsmShareBondProviderInstantiateMsg {
                            owner: "factory_contract".to_string(),
                            factory_contract: "factory_contract".to_string(),
                            transfer_channel_id: "channel-0".to_string(),
                            lsm_redeem_threshold: 0,
                            lsm_redeem_maximum_interval: 0,
                            port_id: "transfer".to_string(),
                            timeout: 0,
                            lsm_min_bond_amount: Uint128::from(0u64),
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes()),
                    }
                )),
                cosmwasm_std::SubMsg::new(cosmwasm_std::CosmosMsg::Wasm(
                    cosmwasm_std::WasmMsg::Instantiate2 {
                        admin: Some("factory_contract".to_string()),
                        code_id: 14,
                        label: "drop-staking-native-bond-provider".to_string(),
                        msg: to_json_binary(&NativeBondProviderInstantiateMsg {
                            owner: "factory_contract".to_string(),
                            base_denom: "base_denom".to_string(),
                            min_stake_amount: Uint128::from(0u64),
                            min_ibc_transfer: Uint128::from(0u64),
                            factory_contract: "factory_contract".to_string(),
                            timeout: 0,
                            transfer_channel_id: "channel-0".to_string(),
                            port_id: "transfer".to_string(),
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: cosmwasm_std::Binary::from("salt".as_bytes()),
                    }
                ))
            ])
            .add_event(
                cosmwasm_std::Event::new("crates.io:drop-staking__drop-factory-instantiate")
                    .add_attributes(vec![
                        cosmwasm_std::attr("action", "init"),
                        cosmwasm_std::attr("base_denom", "base_denom"),
                        cosmwasm_std::attr("sdk_version", "sdk-version"),
                        cosmwasm_std::attr("salt", "salt"),
                        cosmwasm_std::attr(
                            "code_ids",
                            format!(
                                "{:?}",
                                CodeIds {
                                    token_code_id: 1,
                                    core_code_id: 2,
                                    puppeteer_code_id: 3,
                                    withdrawal_voucher_code_id: 5,
                                    withdrawal_manager_code_id: 6,
                                    strategy_code_id: 7,
                                    validators_set_code_id: 8,
                                    distribution_code_id: 9,
                                    rewards_manager_code_id: 10,
                                    rewards_pump_code_id: 11,
                                    splitter_code_id: 12,
                                    lsm_share_bond_provider_code_id: 13,
                                    native_bond_provider_code_id: 14
                                }
                            )
                        ),
                        cosmwasm_std::attr(
                            "remote_opts",
                            format!(
                                "{:?}",
                                RemoteOpts {
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
                                }
                            )
                        ),
                        cosmwasm_std::attr("owner", "owner"),
                        cosmwasm_std::attr("subdenom", "subdenom"),
                        cosmwasm_std::attr(
                            "token_address",
                            "A7AC526557FF685BC20199FC5C34CA4C8F2F5F8ABD4A288D161AD115DC06708A"
                        ),
                        cosmwasm_std::attr(
                            "core_address",
                            "A860CB6CD2DF829AA650D1F195E8A5F4B73BC72DD65FFC2B98D83C8DCB24D3C2"
                        ),
                        cosmwasm_std::attr(
                            "puppeteer_address",
                            "DDB91F7195F3A00683CEF315BF38988FEE4AB7B00518F795E5477EF17928574E"
                        ),
                        cosmwasm_std::attr(
                            "withdrawal_voucher_address",
                            "2EDC9F3CA74FEE8C9C97C5804C5E44B684380772A0C7CFC7FEB14843D437AADD"
                        ),
                        cosmwasm_std::attr(
                            "withdrawal_manager_address",
                            "9080C5A07DA3FAE670586720247E99E9CAB3DF2C79EBD967BEB03747252B97A0"
                        ),
                        cosmwasm_std::attr(
                            "strategy_address",
                            "A611AB41BFB88C7F467A747DFE64B3FE6141D563131D296421B5D4E038C84B9D"
                        ),
                        cosmwasm_std::attr(
                            "validators_set_address",
                            "7B4BE8580FC3E8C59FB57516384CB9E30D3707E560C11C4818D94FE2707A8A6D"
                        ),
                        cosmwasm_std::attr(
                            "distribution_address",
                            "0368B2CE05A215FF095D4B598371205F6CDEAEE8A6332D233E0C6E7F099893AF"
                        ),
                        cosmwasm_std::attr(
                            "rewards_manager_address",
                            "F71DD2183C67E30D7E199C9AB928419B8693183A24C2528790760290895FF118"
                        ),
                        cosmwasm_std::attr(
                            "splitter_address",
                            "99F8E05523E1A4B79E291E7934CC8E306F609D5B470D9D0872925EDCA019D3C1"
                        ),
                        cosmwasm_std::attr(
                            "rewards_pump_address",
                            "6FC09E9FDF411D71139AB50762FB9862D4F519DC21EADB93E93195388E164F25"
                        ),
                        cosmwasm_std::attr(
                            "lsm_share_bond_provider_address",
                            "1D482178B63387218844AA22FC89A3D4465BD4CC07A8DC1232C62EFE4838F955"
                        ),
                        cosmwasm_std::attr(
                            "native_bond_provider_address",
                            "4F119F34DBA97DC2D8E9268BC9513D26CF27BA148EF73DDE924F79A97CA47BBF"
                        ),
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
        transfer_channel_id: None,
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
        transfer_channel_id: Some("channel-1".to_string()),
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
                        tick: true,
                        bond: true,
                        unbond: false,
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
                        tick: true,
                        bond: true,
                        unbond: false,
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
                        tick: true,
                        bond: true,
                        unbond: false,
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
fn test_pause_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        ExecuteMsg::Pause {},
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
fn test_pause() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    set_default_factory_state(deps.as_mut());
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
                        msg: to_json_binary(&CoreExecuteMsg::SetPause(CorePause {
                            tick: true,
                            bond: false,
                            unbond: false,
                        }))
                        .unwrap(),
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
fn test_unpause_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        ExecuteMsg::Unpause {},
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
fn test_unpause() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    let _ = cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    set_default_factory_state(deps.as_mut());
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
                        msg: to_json_binary(&CoreExecuteMsg::SetPause(CorePause {
                            tick: false,
                            bond: false,
                            unbond: false,
                        }))
                        .unwrap(),
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
fn test_query_pause_info() {
    let mut deps = mock_dependencies(&[]);
    deps.querier
        .add_wasm_query_response("core_contract", |_| -> cosmwasm_std::Binary {
            to_json_binary(&CorePause {
                tick: true,
                bond: false,
                unbond: false,
            })
            .unwrap()
        });
    deps.querier.add_wasm_query_response(
        "withdrawal_manager_contract",
        |_| -> cosmwasm_std::Binary {
            to_json_binary(&drop_helpers::pause::PauseInfoResponse::Unpaused {}).unwrap()
        },
    );
    deps.querier
        .add_wasm_query_response("rewards_manager_contract", |_| -> cosmwasm_std::Binary {
            to_json_binary(&drop_helpers::pause::PauseInfoResponse::Paused {}).unwrap()
        });
    set_default_factory_state(deps.as_mut());
    let query_res: drop_staking_base::state::factory::PauseInfoResponse =
        from_json(query(deps.as_ref(), mock_env(), QueryMsg::PauseInfo {}).unwrap()).unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::state::factory::PauseInfoResponse {
            core: CorePause {
                tick: true,
                bond: false,
                unbond: false,
            },
            withdrawal_manager: drop_helpers::pause::PauseInfoResponse::Unpaused {},
            rewards_manager: drop_helpers::pause::PauseInfoResponse::Paused {},
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
