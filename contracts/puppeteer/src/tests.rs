use cosmwasm_std::{
    testing::{mock_env, mock_info},
    Addr, Event, Response,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::puppeteer::Config;
use drop_staking_base::{msg::puppeteer::InstantiateMsg, state::puppeteer::ConfigOptional};

use crate::contract::Puppeteer;

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
