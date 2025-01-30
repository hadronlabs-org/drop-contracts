use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{
    attr, coin,
    testing::{message_info, mock_env},
    to_json_binary, Addr, CosmosMsg, CustomQuery, DepsMut, IbcChannel, IbcEndpoint, IbcOrder,
    Uint128, WasmMsg,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::{
    error::mirror::ContractError,
    state::mirror::{Config, BONDS, CONFIG, COUNTER},
};
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::RequestPacketTimeoutHeight,
};

fn base_init<T>(deps: DepsMut<T>, api: MockApi)
where
    T: CustomQuery,
{
    let config = Config {
        core_contract: api.addr_make("core").to_string(),
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        ibc_timeout: 10,
        prefix: "prefix".to_string(),
    };
    CONFIG.save(deps.storage, &config).unwrap();
    COUNTER.save(deps.storage, &0).unwrap();
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(api.addr_make("owner").as_str()),
    )
    .unwrap();
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("sender"), &[]),
        drop_staking_base::msg::mirror::InstantiateMsg {
            core_contract: api.addr_make("core").to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 10,
            owner: Some(api.addr_make("owner").to_string()),
            prefix: "prefix".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        response,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new("crates.io:drop-staking__drop-mirror-instantiate".to_string())
                .add_attributes(vec![
                    attr("action", "instantiate"),
                    attr("owner", api.addr_make("owner"))
                ])
        )
    );
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            core_contract: api.addr_make("core").to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 10,
            prefix: "prefix".to_string(),
        }
    );
    let owner = cw_ownable::get_ownership(&deps.storage).unwrap();
    assert_eq!(owner.owner, Some(api.addr_make("owner")));
    let counter = COUNTER.load(&deps.storage).unwrap();
    assert_eq!(counter, 0);
}

#[test]
fn test_instantiate_wo_owner() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[]),
        drop_staking_base::msg::mirror::InstantiateMsg {
            core_contract: api.addr_make("core").to_string(),
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
                .add_attributes(vec![
                    attr("action", "instantiate"),
                    attr("owner", api.addr_make("sender"))
                ])
        )
    );
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            core_contract: api.addr_make("core").to_string(),
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            ibc_timeout: 10,
            prefix: "prefix".to_string(),
        }
    );
    let owner = cw_ownable::get_ownership(&deps.storage).unwrap();
    assert_eq!(owner.owner, Some(api.addr_make("sender")));
}

#[test]
fn update_config_ibc_timeout_out_of_range() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    base_init(deps.as_mut(), api);

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateConfig {
            new_config: drop_staking_base::state::mirror::ConfigOptional {
                core_contract: Some(api.addr_make("new_core").to_string()),
                source_port: Some("new_source_port".to_string()),
                source_channel: Some("new_source_channel".to_string()),
                ibc_timeout: Some(u64::MAX),
                prefix: Some("new_prefix".to_string()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(error, ContractError::IbcTimeoutOutOfRange);
}

#[test]
fn update_config_source_channel_not_found() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    base_init(deps.as_mut(), api);

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateConfig {
            new_config: drop_staking_base::state::mirror::ConfigOptional {
                core_contract: Some(api.addr_make("new_core").to_string()),
                source_port: Some("new_source_port".to_string()),
                source_channel: Some("new_source_channel".to_string()),
                ibc_timeout: Some(20),
                prefix: Some("new_prefix".to_string()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(error, ContractError::SourceChannelNotFound);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    base_init(deps.as_mut(), api);

    deps.querier.add_ibc_channel_response(
        Some("channel-0".to_string()),
        Some("transfer".to_string()),
        cosmwasm_std::ChannelResponse::new(Some(IbcChannel::new(
            IbcEndpoint {
                port_id: "port_id".to_string(),
                channel_id: "channel_id".to_string(),
            },
            IbcEndpoint {
                port_id: "port_id".to_string(),
                channel_id: "channel_id".to_string(),
            },
            IbcOrder::Unordered,
            "version".to_string(),
            "connection_id".to_string(),
        ))),
    );
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: api.addr_make("core_contract").to_string(),
                source_channel: "source_channel".to_string(),
                source_port: "source_port".to_string(),
                ibc_timeout: 0u64,
                prefix: "prefix".to_string(),
            },
        )
        .unwrap();
    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateConfig {
            new_config: drop_staking_base::state::mirror::ConfigOptional {
                core_contract: Some(api.addr_make("new_core").to_string()),
                source_port: Some("transfer".to_string()),
                source_channel: Some("channel-0".to_string()),
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
                attr("core_contract", api.addr_make("new_core")),
                attr("ibc_timeout", "20"),
                attr("prefix", "new_prefix"),
                attr("source_port", "transfer"),
                attr("source_channel", "channel-0"),
            ])
        )
    );
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            core_contract: api.addr_make("new_core").to_string(),
            source_port: "transfer".to_string(),
            source_channel: "channel-0".to_string(),
            ibc_timeout: 20,
            prefix: "new_prefix".to_string(),
        }
    );
}

#[test]
fn bond_wrong_receiver() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    base_init(deps.as_mut(), api);

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[coin(1000, "mytoken")]),
        drop_staking_base::msg::mirror::ExecuteMsg::Bond {
            receiver: api.addr_make("some").to_string(),
            r#ref: Some(api.addr_make("reff").to_string()),
            backup: Some(api.addr_make("backup").to_string()),
        },
    );
    assert_eq!(response, Err(ContractError::InvalidPrefix {}));
}

#[test]
fn bond_no_funds() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    base_init(deps.as_mut(), api);

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::Bond {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            r#ref: Some(api.addr_make("reff").to_string()),
            backup: Some(api.addr_make("backup").to_string()),
        },
    );
    assert_eq!(
        response,
        Err(ContractError::PaymentError(
            cw_utils::PaymentError::NoFunds {}
        ))
    );
}

#[test]
fn bond() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    base_init(deps.as_mut(), api);

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("sender"), &[coin(1000, "mytoken")]),
        drop_staking_base::msg::mirror::ExecuteMsg::Bond {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            r#ref: Some(api.addr_make("reff").to_string()),
            backup: Some(api.addr_make("backup").to_string()),
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
            backup: Some(api.addr_make("backup")),
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
                    contract_addr: api.addr_make("core").to_string(),
                    msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Bond {
                        receiver: None,
                        r#ref: Some(api.addr_make("reff").to_string())
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
                        attr("ref", api.addr_make("reff")),
                        attr("backup", api.addr_make("backup")),
                    ])
            )
    );
}

#[test]
fn complete_remote() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });

    base_init(deps.as_mut(), api);

    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(api.addr_make("backup")),
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
        message_info(&api.addr_make("owner"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::Complete { items: vec![1] },
    )
    .unwrap();
    let bond = BONDS.load(&deps.storage, 1).unwrap();
    assert_eq!(
        bond,
        drop_staking_base::state::mirror::BondItem {
            receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                .to_string(),
            backup: Some(api.addr_make("backup")),
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
                    sender: api.addr_make("cosmos2contract").to_string(),
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
    let api = deps.api;

    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });

    base_init(deps.as_mut(), api);

    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(api.addr_make("backup")),
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
        message_info(&api.addr_make("owner"), &[]),
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
                    to_address: api.addr_make("backup").to_string(),
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
    let api = deps.api;

    base_init(deps.as_mut(), api);

    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(api.addr_make("backup")),
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
        message_info(&api.addr_make("owner"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::ChangeReturnType {
            id: 1,
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
        },
    );
    assert_eq!(response, Err(ContractError::Unauthorized {}));

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("backup"), &[]),
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
            backup: Some(api.addr_make("backup")),
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
    let api = deps.api;

    base_init(deps.as_mut(), api);

    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: api.addr_make("receiver").to_string(),
                backup: Some(api.addr_make("backup")),
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
        message_info(&api.addr_make("sender"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateBond {
            id: 1,
            receiver: api.addr_make("new_receiver").to_string(),
            backup: Some(api.addr_make("new_backup").to_string()),
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
        },
    );
    assert_eq!(
        response,
        Err(ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        ))
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("owner"), &[]),
        drop_staking_base::msg::mirror::ExecuteMsg::UpdateBond {
            id: 1,
            receiver: api.addr_make("new_receiver").to_string(),
            backup: Some(api.addr_make("new_backup").to_string()),
            return_type: drop_staking_base::state::mirror::ReturnType::Local,
        },
    )
    .unwrap();

    let bond = BONDS.load(&deps.storage, 1).unwrap();
    assert_eq!(
        bond,
        drop_staking_base::state::mirror::BondItem {
            receiver: api.addr_make("new_receiver").to_string(),
            backup: Some(api.addr_make("new_backup")),
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
                attr("receiver", api.addr_make("new_receiver")),
                attr("backup", api.addr_make("new_backup")),
                attr("return_type", "Local"),
            ])
        )
    );
}

#[test]
fn query_one() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    base_init(deps.as_mut(), api);

    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(api.addr_make("backup")),
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
            backup: Some(api.addr_make("backup")),
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
    let api = deps.api;

    base_init(deps.as_mut(), api);

    BONDS
        .save(
            deps.as_mut().storage,
            1,
            &drop_staking_base::state::mirror::BondItem {
                receiver: "prefix10yaps46wgmzrsslmeqpc9wxpssu7zuw4rrfv8d5rv8pudt8m88446jgnu2j"
                    .to_string(),
                backup: Some(api.addr_make("backup")),
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
                backup: Some(api.addr_make("backup")),
                amount: Uint128::new(1000),
                received: None,
                return_type: drop_staking_base::state::mirror::ReturnType::Remote,
                state: drop_staking_base::state::mirror::BondState::Initiated,
            }
        )])
        .unwrap()
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
        drop_staking_base::msg::mirror::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}

fn get_standard_fees() -> IbcFee {
    IbcFee {
        recv_fee: vec![],
        ack_fee: cosmwasm_std::coins(100, "untrn"),
        timeout_fee: cosmwasm_std::coins(200, "untrn"),
    }
}
