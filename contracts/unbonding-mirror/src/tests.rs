use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, ConfigOptional, CONFIG, UNBOND_REPLY_ID};
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    to_json_binary, Addr, ChannelResponse, Coin, CosmosMsg, Decimal, Decimal256, Event, IbcChannel,
    IbcEndpoint, IbcOrder, OwnedDeps, Response, SubMsg, Timestamp, Uint128, WasmMsg,
};
use drop_helpers::answer::response;
use drop_helpers::testing::mock_dependencies;
use neutron_sdk::{bindings::query::NeutronQuery, interchain_queries::v045::types::Balances};
use std::{collections::HashMap, vec};

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        InstantiateMsg {
            owner: None,
            core_contract: "core_contract".to_string(),
            withdrawal_manager: "withdrawal_manager".to_string(),
            withdrawal_voucher: "withdrawal_voucher".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 12345,
            ibc_denom: "ibc_denom".to_string(),
            prefix: "prefix".to_string(),
            retry_limit: 10,
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-unbonding-mirror-instantiate").add_attributes(
                vec![
                    attr("action", "instantiate"),
                    attr("owner", "owner"),
                    attr("core_contract", "core_contract"),
                    attr("withdrawal_manager", "withdrawal_manager"),
                    attr("withdrawal_voucher", "withdrawal_voucher"),
                    attr("source_port", "source_port"),
                    attr("source_channel", "source_channel"),
                    attr("ibc_timeout", "12345"),
                    attr("ibc_denom", "ibc_denom"),
                    attr("prefix", "prefix"),
                    attr("retry_limit", "10"),
                ]
            )
        )
    );
    assert_eq!(
        CONFIG.load(deps.as_ref().storage).unwrap(),
        Config {
            core_contract: "core_contract".to_string(),
            withdrawal_manager: "withdrawal_manager".to_string(),
            withdrawal_voucher: "withdrawal_voucher".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 12345,
            prefix: "prefix".to_string(),
            ibc_denom: "ibc_denom".to_string(),
            retry_limit: 10,
        }
    );
    assert_eq!(UNBOND_REPLY_ID.load(deps.as_ref().storage).unwrap(), 0u64);
}

#[test]
fn test_execute_update_config_source_channel_not_found() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager1".to_string(),
                withdrawal_voucher: "withdrawal_voucher1".to_string(),
                source_port: "source_port1".to_string(),
                source_channel: "source_channel1".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix1".to_string(),
                ibc_denom: "ibc_denom1".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract2".to_string()),
                withdrawal_manager: Some("withdrawal_manager2".to_string()),
                withdrawal_voucher: Some("withdrawal_voucher2".to_string()),
                source_port: Some("source_port2".to_string()),
                source_channel: Some("source_channel2".to_string()),
                ibc_timeout: Some(54321),
                prefix: Some("prefix2".to_string()),
                ibc_denom: Some("ibc_denom2".to_string()),
                retry_limit: Some(1),
            },
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::SourceChannelNotFound {});
}

#[test]
fn test_execute_update_config_unauthrozied() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("random_sender", &vec![]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract2".to_string()),
                withdrawal_manager: Some("withdrawal_manager2".to_string()),
                withdrawal_voucher: Some("withdrawal_voucher2".to_string()),
                source_port: Some("source_port2".to_string()),
                source_channel: Some("source_channel2".to_string()),
                ibc_timeout: Some(54321),
                prefix: Some("prefix2".to_string()),
                ibc_denom: Some("ibc_denom2".to_string()),
                retry_limit: Some(1),
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
fn test_execute_update_config() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: "core_contract".to_string(),
                withdrawal_manager: "withdrawal_manager1".to_string(),
                withdrawal_voucher: "withdrawal_voucher1".to_string(),
                source_port: "source_port1".to_string(),
                source_channel: "source_channel1".to_string(),
                ibc_timeout: 12345,
                prefix: "prefix1".to_string(),
                ibc_denom: "ibc_denom1".to_string(),
                retry_limit: 10,
            },
        )
        .unwrap();
    deps.querier.add_ibc_channel_response(
        Some("source_channel2".to_string()),
        Some("source_port2".to_string()),
        ChannelResponse {
            channel: Some(IbcChannel::new(
                IbcEndpoint {
                    port_id: "source_port2".to_string(),
                    channel_id: "source_channel2".to_string(),
                },
                IbcEndpoint {
                    port_id: "source_port2".to_string(),
                    channel_id: "source_channel2".to_string(),
                },
                IbcOrder::Ordered,
                "version".to_string(),
                "connection_id".to_string(),
            )),
        },
    );
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &vec![]),
        ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                core_contract: Some("core_contract2".to_string()),
                withdrawal_manager: Some("withdrawal_manager2".to_string()),
                withdrawal_voucher: Some("withdrawal_voucher2".to_string()),
                source_port: Some("source_port2".to_string()),
                source_channel: Some("source_channel2".to_string()),
                ibc_timeout: Some(54321),
                prefix: Some("prefix2".to_string()),
                ibc_denom: Some("ibc_denom2".to_string()),
                retry_limit: Some(1),
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-unbonding-mirror-execute_update_config")
                .add_attributes(vec![
                    attr("action", "execute_update_config"),
                    attr("retry_limit", "1"),
                    attr("core_contract", "core_contract2"),
                    attr("withdrawal_manager", "withdrawal_manager2"),
                    attr("withdrawal_voucher", "withdrawal_voucher2"),
                    attr("ibc_timeout", "54321"),
                    attr("ibc_denom", "ibc_denom2"),
                    attr("prefix", "prefix2"),
                    attr("source_port", "source_port2"),
                    attr("source_channel", "source_channel2"),
                ])
        )
    );
    assert_eq!(
        CONFIG.load(&deps.storage).unwrap(),
        Config {
            core_contract: "core_contract2".to_string(),
            withdrawal_manager: "withdrawal_manager2".to_string(),
            withdrawal_voucher: "withdrawal_voucher2".to_string(),
            source_port: "source_port2".to_string(),
            source_channel: "source_channel2".to_string(),
            ibc_timeout: 54321,
            prefix: "prefix2".to_string(),
            ibc_denom: "ibc_denom2".to_string(),
            retry_limit: 1,
        }
    );
}
