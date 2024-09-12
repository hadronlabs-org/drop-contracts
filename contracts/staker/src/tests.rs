use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Coin, CosmosMsg, Event, Response, SubMsg, Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::staker::{
    Config, ConfigOptional, TxState, CONFIG, ICA, NON_STAKED_BALANCE, TX_STATE,
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
        chain_type: None,
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
        chain_type: None,
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
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();
    let msg = drop_staking_base::msg::staker::ExecuteMsg::IBCTransfer {};
    // no money on the contract
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[Coin::new(600u128, "untrn")]),
        msg.clone(),
    );
    assert_eq!(
        res,
        Err(crate::error::ContractError::InvalidFunds {
            reason: "amount is less than min_ibc_transfer".to_string()
        })
    );

    let mut deps = mock_dependencies(&[Coin::new(10001, "base_denom")]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: coins(100, "local_denom"),
                timeout_fee: coins(200, "local_denom"),
            },
        })
        .unwrap()
    });
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();
    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    ICA.set_address(deps.as_mut().storage, "ica_address", "port", "channel")
        .unwrap();
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[Coin::new(600u128, "untrn")]),
        msg.clone(),
    )
    .unwrap();
    println!("{:?}", res);

    // assert_eq!(
    //     res,
    //     Response::new()
    //         .add_event(
    //             Event::new("crates.io:drop-neutron-contracts__drop-staker-register-ica")
    //                 .add_attributes(vec![
    //                     ("action", "register_ica"),
    //                     ("connection_id", "connection"),
    //                     ("ica_id", "drop_STAKER")
    //                 ])
    //         )
    //         .add_submessage(SubMsg::new(CosmosMsg::Custom(
    //             NeutronMsg::register_interchain_account(
    //                 "connection".to_string(),
    //                 "drop_STAKER".to_string(),
    //                 Some(vec![Coin::new(400u128, "untrn")]),
    //             )
    //         )))
    // );
    // // already asked for registration
    // let res = execute(
    //     deps.as_mut(),
    //     mock_env(),
    //     mock_info("nobody", &[Coin::new(1000u128, "untrn")]),
    //     msg.clone(),
    // );
    // assert_eq!(
    //     res,
    //     Err(crate::error::ContractError::Std(
    //         cosmwasm_std::StdError::generic_err("ICA registration is in progress right now")
    //     ))
    // );
    // // reopen timeouted ICA
    // ICA.set_timeout(deps.as_mut().storage).unwrap();
    // let res = execute(
    //     deps.as_mut(),
    //     mock_env(),
    //     mock_info("nobody", &[Coin::new(1000u128, "untrn")]),
    //     msg.clone(),
    // )
    // .unwrap();
    // assert_eq!(
    //     res,
    //     Response::new()
    //         .add_event(
    //             Event::new("crates.io:drop-neutron-contracts__drop-staker-register-ica")
    //                 .add_attributes(vec![
    //                     ("action", "register_ica"),
    //                     ("connection_id", "connection"),
    //                     ("ica_id", "drop_STAKER")
    //                 ])
    //         )
    //         .add_submessage(SubMsg::new(CosmosMsg::Custom(
    //             NeutronMsg::register_interchain_account(
    //                 "connection".to_string(),
    //                 "drop_STAKER".to_string(),
    //                 Some(vec![Coin::new(400u128, "untrn")]),
    //             )
    //         )))
    // );
}
