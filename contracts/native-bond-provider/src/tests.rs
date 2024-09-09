use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Event,
};
use cw_ownable::Ownership;
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::native_bond_provider::ConfigOptional;

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::native_bond_provider::InstantiateMsg {
            owner: "owner".to_string(),
            base_denom: "base_denom".to_string(),
            staker_contract: "staker_contract".to_string(),
        },
    )
    .unwrap();

    let config = drop_staking_base::state::native_bond_provider::CONFIG
        .load(deps.as_ref().storage)
        .unwrap();

    assert_eq!(
        config,
        drop_staking_base::state::native_bond_provider::Config {
            base_denom: "base_denom".to_string(),
            staker_contract: Addr::unchecked("staker_contract"),
        }
    );

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-native-bond-provider-instantiate")
                .add_attributes([
                    attr("staker_contract", "staker_contract"),
                    attr("base_denom", "base_denom")
                ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    drop_staking_base::state::native_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::native_bond_provider::Config {
                base_denom: "base_denom".to_string(),
                staker_contract: Addr::unchecked("staker_contract"),
            },
        )
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        response,
        to_json_binary(&drop_staking_base::state::native_bond_provider::Config {
            base_denom: "base_denom".to_string(),
            staker_contract: Addr::unchecked("staker_contract"),
        })
        .unwrap()
    );
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::native_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::native_bond_provider::Config {
                base_denom: "base_denom".to_string(),
                staker_contract: Addr::unchecked("staker_contract"),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                base_denom: Some("base_denom".to_string()),
                staker_contract: Some(Addr::unchecked("staker_contract")),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::not_found("type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]")
        ))
    );
}

#[test]
fn update_config_ok() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    drop_staking_base::state::native_bond_provider::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::native_bond_provider::Config {
                base_denom: "base_denom".to_string(),
                staker_contract: Addr::unchecked("staker_contract"),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                base_denom: Some("base_denom_1".to_string()),
                staker_contract: Some(Addr::unchecked("staker_contract_1")),
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        config,
        to_json_binary(&drop_staking_base::state::native_bond_provider::Config {
            base_denom: "base_denom_1".to_string(),
            staker_contract: Addr::unchecked("staker_contract_1"),
        })
        .unwrap()
    );
}

#[test]
fn query_can_process_idle() {
    let deps = mock_dependencies(&[]);

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap();
    assert_eq!(response, to_json_binary(&false).unwrap());
}

#[test]
fn query_ownership() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    )
    .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::Ownership {},
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&Ownership {
            owner: Some(Addr::unchecked("core")),
            pending_owner: None,
            pending_expiry: None
        })
        .unwrap()
    );
}
