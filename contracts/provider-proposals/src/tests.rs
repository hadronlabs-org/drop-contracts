use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, Decimal, Event, OwnedDeps, Querier,
};
use neutron_sdk::bindings::query::NeutronQuery;
use std::marker::PhantomData;

fn mock_dependencies<Q: Querier + Default>() -> OwnedDeps<MockStorage, MockApi, Q, NeutronQuery> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    }
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies::<MockQuerier>();
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        lido_staking_base::msg::validatorset::InstantiateMsg {
            core: "core".to_string(),
            stats_contract: "stats_contract".to_string(),
        },
    )
    .unwrap();

    let config = lido_staking_base::state::validatorset::CONFIG
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(
        config,
        lido_staking_base::state::validatorset::Config {
            core: Addr::unchecked("core"),
            stats_contract: Addr::unchecked("stats_contract"),
        }
    );

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:lido-staking__lido-validators-set-instantiate").add_attributes([
                attr("core", "core"),
                attr("stats_contract", "stats_contract")
            ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies::<MockQuerier>();
    lido_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::validatorset::Config {
                core: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
            },
        )
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::validatorset::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        response,
        to_json_binary(&lido_staking_base::state::validatorset::Config {
            core: Addr::unchecked("core"),
            stats_contract: Addr::unchecked("stats_contract")
        })
        .unwrap()
    );
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::validatorset::Config {
                core: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig {
            core: Some(Addr::unchecked("owner1")),
            stats_contract: Some(Addr::unchecked("stats_contract1")),
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        lido_staking_base::error::validatorset::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::NotFound {
                kind: "type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]".to_string()
            }
        ))
    );
}

#[test]
fn update_config_ok() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    lido_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::validatorset::Config {
                core: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig {
            core: Some(Addr::unchecked("owner1")),
            stats_contract: Some(Addr::unchecked("stats_contract1")),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::validatorset::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        config,
        to_json_binary(&lido_staking_base::state::validatorset::Config {
            core: Addr::unchecked("owner1"),
            stats_contract: Addr::unchecked("stats_contract1")
        })
        .unwrap()
    );
}

#[test]
fn update_validator_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: lido_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        lido_staking_base::error::validatorset::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::NotFound {
                kind: "type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]".to_string()
            }
        ))
    );
}

#[test]
fn update_validator_ok() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: lido_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let validator = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::validatorset::QueryMsg::Validator {
            valoper: Addr::unchecked("valoper_address"),
        },
    )
    .unwrap();
    assert_eq!(
        validator,
        to_json_binary(&lido_staking_base::state::validatorset::ValidatorInfo {
            valoper_address: "valoper_address".to_string(),
            weight: 1,
            last_processed_remote_height: None,
            last_processed_local_height: None,
            last_validated_height: None,
            last_commission_in_range: None,
            uptime: Decimal::zero(),
            tombstone: false,
            jailed_number: None,
        })
        .unwrap()
    );
}

#[test]
fn update_validators_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![lido_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
            }],
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        lido_staking_base::error::validatorset::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::NotFound {
                kind: "type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]".to_string()
            }
        ))
    );
}

#[test]
fn update_validators_ok() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![
                lido_staking_base::msg::validatorset::ValidatorData {
                    valoper_address: "valoper_address1".to_string(),
                    weight: 1,
                },
                lido_staking_base::msg::validatorset::ValidatorData {
                    valoper_address: "valoper_address2".to_string(),
                    weight: 1,
                },
            ],
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let validator = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::validatorset::QueryMsg::Validators {},
    )
    .unwrap();
    assert_eq!(
        validator,
        to_json_binary(&vec![
            lido_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address1".to_string(),
                weight: 1,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
            },
            lido_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address2".to_string(),
                weight: 1,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
            }
        ])
        .unwrap()
    );
}

#[test]
fn update_validator_info_wrong_sender() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    lido_staking_base::state::validatorset::CONFIG
        .save(
            deps_mut.storage,
            &lido_staking_base::state::validatorset::Config {
                core: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
            },
        )
        .unwrap();

    let _response = crate::contract::execute(
        deps_mut,
        mock_env(),
        mock_info("core", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: lido_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
            },
        },
    )
    .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stats_contract1", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidatorsInfo {
            validators: vec![lido_staking_base::msg::validatorset::ValidatorInfoUpdate {
                valoper_address: "valoper_address".to_string(),
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
            }],
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        lido_staking_base::error::validatorset::ContractError::Unauthorized
    );
}

#[test]
fn update_validator_info_ok() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    lido_staking_base::state::validatorset::CONFIG
        .save(
            deps_mut.storage,
            &lido_staking_base::state::validatorset::Config {
                core: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: lido_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stats_contract", &[]),
        lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidatorsInfo {
            validators: vec![lido_staking_base::msg::validatorset::ValidatorInfoUpdate {
                valoper_address: "valoper_address".to_string(),
                last_processed_remote_height: Some(1234),
                last_processed_local_height: Some(2345),
                last_validated_height: Some(3456),
                last_commission_in_range: Some(4567),
                uptime: Decimal::one(),
                tombstone: true,
                jailed_number: Some(5678),
            }],
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let validator = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::validatorset::QueryMsg::Validator {
            valoper: Addr::unchecked("valoper_address"),
        },
    )
    .unwrap();

    assert_eq!(
        validator,
        to_json_binary(&lido_staking_base::state::validatorset::ValidatorInfo {
            valoper_address: "valoper_address".to_string(),
            weight: 1,
            last_processed_remote_height: Some(1234),
            last_processed_local_height: Some(2345),
            last_validated_height: Some(3456),
            last_commission_in_range: Some(4567),
            uptime: Decimal::one(),
            tombstone: true,
            jailed_number: Some(5678),
        })
        .unwrap()
    );
}
