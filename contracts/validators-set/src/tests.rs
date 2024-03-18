use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MockQuerier},
    to_json_binary, Addr, Decimal, Event,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::validatorset::ConfigOptional;

#[test]
fn instantiate() {
    let mut deps = mock_dependencies::<MockQuerier>();
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        drop_staking_base::msg::validatorset::InstantiateMsg {
            owner: "owner".to_string(),
            stats_contract: "stats_contract".to_string(),
        },
    )
    .unwrap();

    let config = drop_staking_base::state::validatorset::CONFIG
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(
        config,
        drop_staking_base::state::validatorset::Config {
            owner: Addr::unchecked("owner"),
            stats_contract: Addr::unchecked("stats_contract"),
            provider_proposals_contract: None,
        }
    );

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-validators-set-instantiate").add_attributes([
                attr("owner", "owner"),
                attr("stats_contract", "stats_contract")
            ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies::<MockQuerier>();
    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                owner: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
            },
        )
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::validatorset::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        response,
        to_json_binary(&drop_staking_base::state::validatorset::Config {
            owner: Addr::unchecked("core"),
            stats_contract: Addr::unchecked("stats_contract"),
            provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract"))
        })
        .unwrap()
    );
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                owner: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                owner: Some(Addr::unchecked("owner1")),
                stats_contract: Some(Addr::unchecked("stats_contract1")),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract1")),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::validatorset::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::not_found("type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]")
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

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                owner: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                owner: Some(Addr::unchecked("owner1")),
                stats_contract: Some(Addr::unchecked("stats_contract1")),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract1")),
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::validatorset::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        config,
        to_json_binary(&drop_staking_base::state::validatorset::Config {
            owner: Addr::unchecked("owner1"),
            stats_contract: Addr::unchecked("stats_contract1"),
            provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract1"))
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
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: drop_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::validatorset::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::not_found("type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]")
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
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: drop_staking_base::msg::validatorset::ValidatorData {
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
        drop_staking_base::msg::validatorset::QueryMsg::Validator {
            valoper: "valoper_address".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        validator,
        to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
            validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0,
                total_voted_proposals: 0,
            })
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
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![drop_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
            }],
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::validatorset::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
            cosmwasm_std::StdError::not_found("type: cw_ownable::Ownership<cosmwasm_std::addresses::Addr>; key: [6F, 77, 6E, 65, 72, 73, 68, 69, 70]")
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
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![
                drop_staking_base::msg::validatorset::ValidatorData {
                    valoper_address: "valoper_address1".to_string(),
                    weight: 1,
                },
                drop_staking_base::msg::validatorset::ValidatorData {
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
        drop_staking_base::msg::validatorset::QueryMsg::Validators {},
    )
    .unwrap();
    assert_eq!(
        validator,
        to_json_binary(&vec![
            drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address1".to_string(),
                weight: 1,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0,
                total_voted_proposals: 0,
            },
            drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address2".to_string(),
                weight: 1,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0,
                total_voted_proposals: 0,
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

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps_mut.storage,
            &drop_staking_base::state::validatorset::Config {
                owner: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
            },
        )
        .unwrap();

    let _response = crate::contract::execute(
        deps_mut,
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: drop_staking_base::msg::validatorset::ValidatorData {
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
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidatorsInfo {
            validators: vec![drop_staking_base::msg::validatorset::ValidatorInfoUpdate {
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
        drop_staking_base::error::validatorset::ContractError::Unauthorized
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

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps_mut.storage,
            &drop_staking_base::state::validatorset::Config {
                owner: Addr::unchecked("core"),
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator {
            validator: drop_staking_base::msg::validatorset::ValidatorData {
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
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidatorsInfo {
            validators: vec![drop_staking_base::msg::validatorset::ValidatorInfoUpdate {
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
        drop_staking_base::msg::validatorset::QueryMsg::Validator {
            valoper: "valoper_address".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        validator,
        to_json_binary(&drop_staking_base::msg::validatorset::ValidatorResponse {
            validator: Some(drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
                last_processed_remote_height: Some(1234),
                last_processed_local_height: Some(2345),
                last_validated_height: Some(3456),
                last_commission_in_range: Some(4567),
                uptime: Decimal::one(),
                tombstone: true,
                jailed_number: Some(5678),
                init_proposal: None,
                total_passed_proposals: 0,
                total_voted_proposals: 0,
            })
        })
        .unwrap()
    );
}
