use cosmwasm_std::{
    attr, coin,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, CosmosMsg, CustomQuery, DepsMut, Uint128, WasmMsg,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::mirror::{BONDS, CONFIG, COUNTER};
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::RequestPacketTimeoutHeight,
};

fn base_init<T>(deps: DepsMut<T>)
where
    T: CustomQuery,
{
    let config = drop_staking_base::state::mirror::Config {
        core_contract: "core".to_string(),
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        ibc_timeout: 10,
        prefix: "prefix".to_string(),
    };
    CONFIG.save(deps.storage, &config).unwrap();
    COUNTER.save(deps.storage, &0).unwrap();
    cw_ownable::initialize_owner(deps.storage, deps.api, Some("owner")).unwrap();
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        drop_staking_base::msg::mirror::InstantiateMsg {
            core_contract: "core".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 10,
            owner: Some("owner".to_string()),
            prefix: "prefix".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        response,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new("crates.io:drop-staking__drop-mirror-instantiate".to_string())
                .add_attributes(vec![attr("action", "instantiate"), attr("owner", "owner")])
        )
    );
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        config,
        drop_staking_base::state::mirror::Config {
            core_contract: "core".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 10,
            prefix: "prefix".to_string(),
        }
    );
    let owner = cw_ownable::get_ownership(&deps.storage).unwrap();
    assert_eq!(owner.owner, Some(Addr::unchecked("owner")));
    let counter = COUNTER.load(&deps.storage).unwrap();
    assert_eq!(counter, 0);
}

#[test]
fn test_instantiate_wo_owner() {
    let mut deps = mock_dependencies(&[]);
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        drop_staking_base::msg::mirror::InstantiateMsg {
            core_contract: "core".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 10,
            owner: None,
            prefix: "prefix".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        response,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new("crates.io:drop-staking__drop-mirror-instantiate".to_string())
                .add_attributes(vec![attr("action", "instantiate"), attr("owner", "sender")])
        )
    );
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        config,
        drop_staking_base::state::mirror::Config {
            core_contract: "core".to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 10,
            prefix: "prefix".to_string(),
        }
    );
    let owner = cw_ownable::get_ownership(&deps.storage).unwrap();
    assert_eq!(owner.owner, Some(Addr::unchecked("sender")));
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateConfig {
            new_config: drop_staking_base::state::mirror::ConfigOptional {
                core_contract: Some("new_core".to_string()),
                source_port: Some("new_source_port".to_string()),
                source_channel: Some("new_source_channel".to_string()),
                ibc_timeout: Some(20),
                prefix: Some("new_prefix".to_string()),
            },
        },
    )
    .unwrap();
    assert_eq!(
        response,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-staking__drop-mirror-update_config".to_string()
            )
            .add_attributes(vec![
                attr("action", "update_config"),
                attr("core_contract", "new_core"),
                attr("source_port", "new_source_port"),
                attr("source_channel", "new_source_channel"),
                attr("ibc_timeout", "20"),
                attr("prefix", "new_prefix"),
            ])
        )
    );
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        config,
        drop_staking_base::state::mirror::Config {
            core_contract: "new_core".to_string(),
            source_port: "new_source_port".to_string(),
            source_channel: "new_source_channel".to_string(),
            ibc_timeout: 20,
            prefix: "new_prefix".to_string(),
        }
    );
}

#[test]
fn bond_wrong_receiver() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[coin(1000, "mytoken")]),
        drop_staking_base::msg::mirror::ExecuteMsg::Bond {
            receiver: "some".to_string(),
            r#ref: Some("reff".to_string()),
            backup: Some("backup".to_string()),
        },
    );
    assert_eq!(
        response,
        Err(drop_staking_base::error::mirror::ContractError::InvalidPrefix {})
    );
}

#[test]
fn bond_no_funds() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::Bond {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            r#ref: Some("reff".to_string()),
            backup: Some("backup".to_string()),
        },
    );
    assert_eq!(
        response,
        Err(
            drop_staking_base::error::mirror::ContractError::PaymentError(
                cw_utils::PaymentError::NoFunds {}
            )
        )
    );
}

#[test]
fn bond() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[coin(1000, "mytoken")]),
        drop_staking_base::msg::mirror::ExecuteMsg::Bond {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            r#ref: Some("reff".to_string()),
            backup: Some("backup".to_string()),
        },
    )
    .unwrap();
    let counter = COUNTER.load(&deps.storage).unwrap();
    assert_eq!(counter, 1);
    let bond = BONDS.load(&deps.storage, 1).unwrap();
    assert_eq!(
        bond,
        drop_staking_base::state::mirror::BondItem {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            backup: Some(Addr::unchecked("backup".to_string())),
            amount: Uint128::new(1000),
            received: None,
            return_type: drop_staking_base::state::mirror::ReturnType::Remote,
            state: drop_staking_base::state::mirror::BondState::Initiated,
        }
    );
    assert_eq!(
        response,
        cosmwasm_std::Response::new()
            .add_submessage(cosmwasm_std::SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: "core".to_string(),
                    msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Bond {
                        receiver: None,
                        r#ref: Some("reff".to_string())
                    })
                    .unwrap(),
                    funds: vec![coin(1000, "mytoken")],
                },
                1
            ))
            .add_event(
                cosmwasm_std::Event::new("crates.io:drop-staking__drop-mirror-bond".to_string())
                    .add_attributes(vec![
                        attr("action", "bond"),
                        attr("id", "1"),
                        attr(
                            "receiver",
                            "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                        ),
                        attr("ref", "reff"),
                        attr("backup", "backup"),
                    ])
            )
    );
}

#[test]
fn complete_remote() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(deps.as_mut());
    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(Addr::unchecked("backup".to_string())),
                amount: Uint128::new(1000),
                received: Some(coin(1000, "ld_denom")),
                return_type: drop_staking_base::state::mirror::ReturnType::Remote,
                state: drop_staking_base::state::mirror::BondState::Bonded,
            },
        )
        .unwrap();
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::Complete { items: vec![1] },
    )
    .unwrap();
    let bond = BONDS.load(&deps.storage, 1).unwrap();
    assert_eq!(
        bond,
        drop_staking_base::state::mirror::BondItem {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            backup: Some(Addr::unchecked("backup".to_string())),
            amount: Uint128::new(1000),
            received: Some(coin(1000, "ld_denom")),
            return_type: drop_staking_base::state::mirror::ReturnType::Remote,
            state: drop_staking_base::state::mirror::BondState::Sent,
        }
    );
    assert_eq!(
        response,
        cosmwasm_std::Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(CosmosMsg::Custom(
                NeutronMsg::IbcTransfer {
                    source_port: "source_port".to_string(),
                    source_channel: "source_channel".to_string(),
                    token: coin(1000, "ld_denom"),
                    sender: "cosmos2contract".to_string(),
                    receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                        .to_string(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None
                    },
                    timeout_timestamp: 1571797429879305533,
                    memo: "1".to_string(),
                    fee: get_standard_fees(),
                }
            )))
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-staking__drop-mirror-complete".to_string()
                )
                .add_attributes(vec![
                    attr("action", "complete"),
                    attr("id", "1"),
                    attr("return_type", "Bonded"),
                    attr("coin", "1000ld_denom"),
                ])
            )
    );
}

#[test]
fn complete_local() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(deps.as_mut());
    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(Addr::unchecked("backup".to_string())),
                amount: Uint128::new(1000),
                received: Some(coin(1000, "ld_denom")),
                return_type: drop_staking_base::state::mirror::ReturnType::Local,
                state: drop_staking_base::state::mirror::BondState::Bonded,
            },
        )
        .unwrap();
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::Complete { items: vec![1] },
    )
    .unwrap();
    let bond = BONDS.load(&deps.storage, 1);
    assert!(bond.is_err());
    assert_eq!(
        response,
        cosmwasm_std::Response::new()
            .add_submessage(cosmwasm_std::SubMsg::new(CosmosMsg::Bank(
                cosmwasm_std::BankMsg::Send {
                    to_address: "backup".to_string(),
                    amount: vec![coin(1000, "ld_denom")],
                }
            )))
            .add_event(
                cosmwasm_std::Event::new(
                    "crates.io:drop-staking__drop-mirror-complete".to_string()
                )
                .add_attributes(vec![
                    attr("action", "complete"),
                    attr("id", "1"),
                    attr("return_type", "Bonded"),
                    attr("coin", "1000ld_denom"),
                ])
            )
    );
}

#[test]
fn change_return_type() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(Addr::unchecked("backup".to_string())),
                amount: Uint128::new(1000),
                received: Some(coin(1000, "ld_denom")),
                return_type: drop_staking_base::state::mirror::ReturnType::Remote,
                state: drop_staking_base::state::mirror::BondState::Bonded,
            },
        )
        .unwrap();
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::ChangeReturnType {
            id: 1,
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
        },
    );
    assert_eq!(
        response,
        Err(drop_staking_base::error::mirror::ContractError::Unauthorized {})
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("backup", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::ChangeReturnType {
            id: 1,
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
        },
    )
    .unwrap();

    let bond = BONDS.load(&deps.storage, 1).unwrap();
    assert_eq!(
        bond,
        drop_staking_base::state::mirror::BondItem {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            backup: Some(Addr::unchecked("backup".to_string())),
            amount: Uint128::new(1000),
            received: Some(coin(1000, "ld_denom")),
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
            state: drop_staking_base::state::mirror::BondState::Bonded,
        }
    );
    assert_eq!(
        response,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-staking__drop-mirror-change_return_type".to_string()
            )
            .add_attributes(vec![
                attr("action", "change_return_type"),
                attr("id", "1"),
                attr("return_type", "Local"),
            ])
        )
    );
}

#[test]
fn update_bond() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "receiver".to_string(),
                backup: Some(Addr::unchecked("backup".to_string())),
                amount: Uint128::new(1000),
                received: None,
                return_type: drop_staking_base::state::mirror::ReturnType::Remote,
                state: drop_staking_base::state::mirror::BondState::Initiated,
            },
        )
        .unwrap();
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateBond {
            id: 1,
            receiver: "new_receiver".to_string(),
            backup: Some("new_backup".to_string()),
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
        },
    );
    assert_eq!(
        response,
        Err(
            drop_staking_base::error::mirror::ContractError::OwnershipError(
                cw_ownable::OwnershipError::NotOwner
            )
        )
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateBond {
            id: 1,
            receiver: "new_receiver".to_string(),
            backup: Some("new_backup".to_string()),
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
        },
    )
    .unwrap();

    let bond = BONDS.load(&deps.storage, 1).unwrap();
    assert_eq!(
        bond,
        drop_staking_base::state::mirror::BondItem {
            receiver: "new_receiver".to_string(),
            backup: Some(Addr::unchecked("new_backup".to_string())),
            amount: Uint128::new(1000),
            received: None,
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
            state: drop_staking_base::state::mirror::BondState::Initiated,
        }
    );
    assert_eq!(
        response,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-staking__drop-mirror-update_bond_state".to_string()
            )
            .add_attributes(vec![
                attr("action", "update_bond"),
                attr("id", "1"),
                attr("receiver", "new_receiver"),
                attr("backup", "new_backup"),
                attr("return_type", "Local"),
            ])
        )
    );
}

#[test]
fn query_one() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(Addr::unchecked("backup".to_string())),
                amount: Uint128::new(1000),
                received: None,
                return_type: drop_staking_base::state::mirror::ReturnType::Remote,
                state: drop_staking_base::state::mirror::BondState::Initiated,
            },
        )
        .unwrap();
    let one = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::mirror::QueryMsg::One { id: 1 },
    )
    .unwrap();
    assert_eq!(
        one,
        to_json_binary(&drop_staking_base::state::mirror::BondItem {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            backup: Some(Addr::unchecked("backup".to_string())),
            amount: Uint128::new(1000),
            received: None,
            return_type: drop_staking_base::state::mirror::ReturnType::Remote,
            state: drop_staking_base::state::mirror::BondState::Initiated,
        })
        .unwrap()
    );
}

#[test]
fn query_all() {
    let mut deps = mock_dependencies(&[]);
    base_init(deps.as_mut());
    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(Addr::unchecked("backup".to_string())),
                amount: Uint128::new(1000),
                received: None,
                return_type: drop_staking_base::state::mirror::ReturnType::Remote,
                state: drop_staking_base::state::mirror::BondState::Initiated,
            },
        )
        .unwrap();
    let all = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::mirror::QueryMsg::All {
            limit: None,
            start_after: None,
        },
    )
    .unwrap();
    assert_eq!(
        all,
        to_json_binary(&vec![(
            1,
            drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(Addr::unchecked("backup".to_string())),
                amount: Uint128::new(1000),
                received: None,
                return_type: drop_staking_base::state::mirror::ReturnType::Remote,
                state: drop_staking_base::state::mirror::BondState::Initiated,
            }
        )])
        .unwrap()
    );
}

fn get_standard_fees() -> IbcFee {
    IbcFee {
        recv_fee: vec![],
        ack_fee: cosmwasm_std::coins(100, "untrn"),
        timeout_fee: cosmwasm_std::coins(200, "untrn"),
    }
}
