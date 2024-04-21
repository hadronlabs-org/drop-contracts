use cosmwasm_std::{
    testing::{mock_env, mock_info},
    Addr, Coin, CosmosMsg, Event, Response, SubMsg, Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::staker::{
    Config, ConfigOptional, TxState, CONFIG, ICA, NON_STAKED_BALANCE, TX_STATE,
};
use neutron_sdk::bindings::msg::NeutronMsg;
// use prost::Message;

use crate::contract::{execute, instantiate};

fn get_default_config() -> Config {
    Config {
        connection_id: "connection".to_string(),
        ibc_fees: drop_helpers::interchain::IBCFees {
            recv_fee: Uint128::from(100u128),
            ack_fee: Uint128::from(200u128),
            timeout_fee: Uint128::from(300u128),
            register_fee: Uint128::from(400u128),
        },
        timeout: 10u64,
        port_id: "port_id".to_string(),
        transfer_channel_id: "transfer_channel_id".to_string(),
        remote_denom: "remote_denom".to_string(),
        base_denom: "base_denom".to_string(),
        allowed_senders: vec!["core".to_string()],
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
        ibc_fees: drop_helpers::interchain::IBCFees {
            recv_fee: Uint128::from(100u128),
            ack_fee: Uint128::from(200u128),
            timeout_fee: Uint128::from(300u128),
            register_fee: Uint128::from(400u128),
        },
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
            ("msg", "InstantiateMsg { connection_id: \"connection\", port_id: \"port_id\", ibc_fees: IBCFees { recv_fee: Uint128(100), ack_fee: Uint128(200), timeout_fee: Uint128(300), register_fee: Uint128(400) }, timeout: 10, remote_denom: \"remote_denom\", base_denom: \"base_denom\", transfer_channel_id: \"transfer_channel_id\", owner: Some(\"owner\"), allowed_senders: [\"core\"], min_ibc_transfer: Uint128(10000), min_staking_amount: Uint128(10000) }"),
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
        ibc_fees: drop_helpers::interchain::IBCFees {
            recv_fee: Uint128::from(100u128),
            ack_fee: Uint128::from(200u128),
            timeout_fee: Uint128::from(300u128),
            register_fee: Uint128::from(400u128),
        },
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
        connection_id: Some("new_connection".to_string()),
        ibc_fees: Some(drop_helpers::interchain::IBCFees {
            recv_fee: Uint128::from(1100u128),
            ack_fee: Uint128::from(1200u128),
            timeout_fee: Uint128::from(1300u128),
            register_fee: Uint128::from(1400u128),
        }),
        timeout: Some(20u64),
        port_id: Some("new_port_id".to_string()),
        transfer_channel_id: Some("new_transfer_channel_id".to_string()),
        remote_denom: Some("new_remote_denom".to_string()),
        base_denom: Some("new_base_denom".to_string()),
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
            ("new_config", "ConfigOptional { port_id: Some(\"new_port_id\"), transfer_channel_id: Some(\"new_transfer_channel_id\"), connection_id: Some(\"new_connection\"), ibc_fees: Some(IBCFees { recv_fee: Uint128(1100), ack_fee: Uint128(1200), timeout_fee: Uint128(1300), register_fee: Uint128(1400) }), timeout: Some(20), remote_denom: Some(\"new_remote_denom\"), base_denom: Some(\"new_base_denom\"), allowed_senders: Some([\"new_core\"]), puppeteer_ica: Some(\"puppeteer_ica\"), min_ibc_transfer: Some(Uint128(110000)), min_staking_amount: Some(Uint128(110000)) }")
        ]))
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            connection_id: "new_connection".to_string(),
            ibc_fees: drop_helpers::interchain::IBCFees {
                recv_fee: Uint128::from(1100u128),
                ack_fee: Uint128::from(1200u128),
                timeout_fee: Uint128::from(1300u128),
                register_fee: Uint128::from(1400u128),
            },
            timeout: 20u64,
            port_id: "new_port_id".to_string(),
            transfer_channel_id: "new_transfer_channel_id".to_string(),
            remote_denom: "new_remote_denom".to_string(),
            base_denom: "new_base_denom".to_string(),
            allowed_senders: vec!["new_core".to_string()],
            puppeteer_ica: Some("puppeteer_ica".to_string()),
            min_ibc_transfer: Uint128::from(110000u128),
            min_staking_amount: Uint128::from(110000u128),
        }
    );
}

#[test]
fn test_register_ica() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let msg = drop_staking_base::msg::staker::ExecuteMsg::RegisterICA {};
    // no fees
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[]),
        msg.clone(),
    );
    assert_eq!(
        res,
        Err(crate::error::ContractError::PaymentError(
            cw_utils::PaymentError::NoFunds {}
        ))
    );

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[Coin::new(1000u128, "untrn")]),
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
                    Some(vec![Coin::new(400u128, "untrn")]),
                )
            )))
    );
    // already asked for registration
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[Coin::new(1000u128, "untrn")]),
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
        mock_info("nobody", &[Coin::new(1000u128, "untrn")]),
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
                    Some(vec![Coin::new(400u128, "untrn")]),
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
    // no fees
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[]),
        msg.clone(),
    );
    assert_eq!(
        res,
        Err(crate::error::ContractError::PaymentError(
            cw_utils::PaymentError::NoFunds {}
        ))
    );
    // low fees
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &[Coin::new(100u128, "untrn")]),
        msg.clone(),
    );
    assert_eq!(
        res,
        Err(crate::error::ContractError::InvalidFunds {
            reason: "invalid amount: expected at least 600, got 100".to_string()
        })
    );
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
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();
    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();
    ICA.set_address(deps.as_mut().storage, "ica_address")
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
