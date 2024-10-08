use crate::{
    contract::{execute, instantiate},
    error::ContractError,
};
use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Event, Response, SubMsg,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::pump::{Config, CONFIG, ICA};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        types::ProtobufAny,
    },
    query::min_ibc_fee::MinIbcFeeResponse,
};
use prost::Message;

fn get_default_config() -> Config {
    Config {
        dest_address: Some(Addr::unchecked("dest_address")),
        dest_channel: Some("dest_channel".to_string()),
        dest_port: Some("dest_port".to_string()),
        connection_id: "connection".to_string(),
        refundee: Some(Addr::unchecked("refundee")),
        timeout: drop_staking_base::state::pump::PumpTimeout {
            local: Some(10u64),
            remote: 10u64,
        },
        local_denom: "local_denom".to_string(),
    }
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = drop_staking_base::msg::pump::InstantiateMsg {
        dest_address: Some("dest_address".to_string()),
        dest_channel: Some("dest_channel".to_string()),
        dest_port: Some("dest_port".to_string()),
        connection_id: "connection".to_string(),
        refundee: Some("refundee".to_string()),
        timeout: drop_staking_base::state::pump::PumpTimeout {
            local: Some(10u64),
            remote: 10u64,
        },
        local_denom: "local_denom".to_string(),
        owner: Some("owner".to_string()),
    };
    let res = instantiate(deps.as_mut(), mock_env(), mock_info("admin", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-neutron-contracts__drop-pump-instantiate"
        ).add_attributes(vec![
            ("contract_name", "crates.io:drop-neutron-contracts__drop-pump"),
            ("contract_version", "1.0.0"),
            ("msg", "InstantiateMsg { dest_address: Some(\"dest_address\"), dest_channel: Some(\"dest_channel\"), dest_port: Some(\"dest_port\"), connection_id: \"connection\", refundee: Some(\"refundee\"), timeout: PumpTimeout { local: Some(10), remote: 10 }, local_denom: \"local_denom\", owner: Some(\"owner\") }"),
            ("sender", "admin")
        ]))
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, get_default_config());
    let owner = cw_ownable::get_ownership(deps.as_ref().storage)
        .unwrap()
        .owner
        .unwrap();
    assert_eq!(owner, Addr::unchecked("owner"));
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = drop_staking_base::msg::pump::InstantiateMsg {
        dest_address: Some("dest_address".to_string()),
        dest_channel: Some("dest_channel".to_string()),
        dest_port: Some("dest_port".to_string()),
        connection_id: "connection".to_string(),
        refundee: Some("refundee".to_string()),
        timeout: drop_staking_base::state::pump::PumpTimeout {
            local: Some(0u64),
            remote: 0u64,
        },
        local_denom: "local_denom".to_string(),
        owner: Some("owner".to_string()),
    };
    let _res = instantiate(deps.as_mut(), mock_env(), mock_info("admin", &[]), msg).unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("admin")).unwrap();
    let msg = drop_staking_base::msg::pump::UpdateConfigMsg {
        dest_address: Some("new_dest_address".to_string()),
        dest_channel: Some("new_dest_channel".to_string()),
        dest_port: Some("new_dest_port".to_string()),
        connection_id: Some("new_connection".to_string()),
        refundee: Some("new_refundee".to_string()),
        timeout: Some(drop_staking_base::state::pump::PumpTimeout {
            local: Some(1u64),
            remote: 1u64,
        }),
        local_denom: Some("new_local_denom".to_string()),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::pump::ExecuteMsg::UpdateConfig {
            new_config: Box::new(msg),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(Event::new(
            "crates.io:drop-neutron-contracts__drop-pump-update_config"
        ).add_attributes(vec![
            ("action","update_config"),
            ("new_config", "UpdateConfigMsg { dest_address: Some(\"new_dest_address\"), dest_channel: Some(\"new_dest_channel\"), dest_port: Some(\"new_dest_port\"), connection_id: Some(\"new_connection\"), refundee: Some(\"new_refundee\"), timeout: Some(PumpTimeout { local: Some(1), remote: 1 }), local_denom: Some(\"new_local_denom\") }")
        ]))
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            dest_address: Some(Addr::unchecked("new_dest_address")),
            dest_channel: Some("new_dest_channel".to_string()),
            dest_port: Some("new_dest_port".to_string()),
            connection_id: "new_connection".to_string(),
            refundee: Some(Addr::unchecked("new_refundee")),
            timeout: drop_staking_base::state::pump::PumpTimeout {
                local: Some(1u64),
                remote: 1u64,
            },
            local_denom: "new_local_denom".to_string(),
        }
    );
}

#[test]
fn test_register_ica_no_fee() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let msg = drop_staking_base::msg::pump::ExecuteMsg::RegisterICA {};

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
            reason: "missing fee in denom local_denom".to_string()
        }
    )
}

#[test]
fn test_register_ica() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let msg = drop_staking_base::msg::pump::ExecuteMsg::RegisterICA {};

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &coins(1000, "local_denom")),
        msg.clone(),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-register-ica")
                    .add_attributes(vec![
                        ("action", "register_ica"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_PUMP")
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Custom(
                NeutronMsg::register_interchain_account(
                    "connection".to_string(),
                    "drop_PUMP".to_string(),
                    Some(coins(1000, "local_denom")),
                )
            )))
    );
    // already asked for registration
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("nobody", &coins(1000, "local_denom")),
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
        mock_info("nobody", &coins(1000, "local_denom")),
        msg.clone(),
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-register-ica")
                    .add_attributes(vec![
                        ("action", "register_ica"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_PUMP")
                    ])
            )
            .add_submessage(SubMsg::new(CosmosMsg::Custom(
                NeutronMsg::register_interchain_account(
                    "connection".to_string(),
                    "drop_PUMP".to_string(),
                    Some(coins(1000, "local_denom")),
                )
            )))
    );
}

#[test]
fn test_execute_refund_no_refundee() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    let mut config = get_default_config();
    config.refundee = None;
    CONFIG.save(deps.as_mut().storage, &config).unwrap();
    let err = execute(deps.as_mut(), mock_env(), mock_info("nobody", &[]), msg).unwrap_err();
    assert_eq!(err, ContractError::RefundeeIsNotSet {});
}

#[test]
fn test_execute_refund_success_refundee() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let res = execute(deps.as_mut(), mock_env(), mock_info("refundee", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-refund")
                    .add_attributes(vec![("action", "refund"), ("refundee", "refundee")])
            )
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "refundee".to_string(),
                amount: vec![Coin::new(200, "untrn")]
            }))
    );
}

#[test]
fn test_execute_refund_success_owner() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    {
        let deps = deps.as_mut();
        cw_ownable::initialize_owner(deps.storage, deps.api, Some("owner")).unwrap();
    }

    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-refund")
                    .add_attributes(vec![("action", "refund"), ("refundee", "refundee")])
            )
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: "refundee".to_string(),
                amount: vec![Coin::new(200, "untrn")]
            }))
    );
}

#[test]
fn test_execute_refund_permission_denied() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Refund {
        coins: coins(200, "untrn"),
    };
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    {
        let deps = deps.as_mut();
        cw_ownable::initialize_owner(deps.storage, deps.api, Some("owner")).unwrap();
    }

    let err = execute(deps.as_mut(), mock_env(), mock_info("nobody", &[]), msg).unwrap_err();
    assert_eq!(err, crate::error::ContractError::Unauthorized {});
}

#[test]
fn test_push() {
    let msg = drop_staking_base::msg::pump::ExecuteMsg::Push {
        coins: vec![Coin::new(100u128, "remote_denom")],
    };
    let mut deps = mock_dependencies(&[]);
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
    ICA.set_address(deps.as_mut().storage, "some", "port", "channel")
        .unwrap();
    CONFIG
        .save(deps.as_mut().storage, &get_default_config())
        .unwrap();
    let env = mock_env();
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("nobody", &[]),
        msg.clone(),
    )
    .unwrap();
    let msg = cosmos_sdk_proto::ibc::applications::transfer::v1::MsgTransfer {
        source_port: "dest_port".to_string(),
        source_channel: "dest_channel".to_string(),
        token: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: "remote_denom".to_string(),
            amount: "100".to_string(),
        }),
        sender: "some".to_string(),
        receiver: "dest_address".to_string(),
        timeout_height: None,
        timeout_timestamp: env.block.time.plus_seconds(10).nanos(),
    };
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    let any_msg = ProtobufAny {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: Binary::from(buf),
    };
    assert_eq!(
        res,
        Response::new()
            .add_event(
                Event::new("crates.io:drop-neutron-contracts__drop-pump-push").add_attributes(
                    vec![
                        ("action", "push"),
                        ("connection_id", "connection"),
                        ("ica_id", "drop_PUMP"),
                        ("coins", "[Coin { 100 \"remote_denom\" }]"),
                    ]
                )
            )
            .add_message(CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection".to_string(),
                "drop_PUMP".to_string(),
                vec![any_msg],
                "".to_string(),
                10u64,
                IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(100, "local_denom"),
                    timeout_fee: coins(200, "local_denom")
                }
            )))
    );
}
