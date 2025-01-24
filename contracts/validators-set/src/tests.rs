use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env},
    to_json_binary, Addr, Decimal, Event, Response, Uint128,
};
use cosmwasm_std::testing::message_info;
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::{provider_proposals::ProposalInfo, validatorset::ConfigOptional};

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("admin"), &[]),
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
            stats_contract: Addr::unchecked("stats_contract"),
            provider_proposals_contract: None,
            val_ref_contract: None,
        }
    );

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-validators-set-instantiate")
                .add_attributes([attr("stats_contract", "stats_contract")])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
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
            stats_contract: Addr::unchecked("stats_contract"),
            provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
            val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
        })
        .unwrap()
    );
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core1"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                stats_contract: Some("stats_contract1".to_string()),
                provider_proposals_contract: Some("provider_proposals_contract1".to_string()),
                val_ref_contract: Some("val_ref_contract1".to_string()),
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
    let mut deps = mock_dependencies(&[]);

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
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                stats_contract: Some("stats_contract1".to_string()),
                provider_proposals_contract: Some("provider_proposals_contract1".to_string()),
                val_ref_contract: Some("val_ref_contract1".to_string()),
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
            stats_contract: Addr::unchecked("stats_contract1"),
            provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract1")),
            val_ref_contract: Some(Addr::unchecked("val_ref_contract1")),
        })
        .unwrap()
    );
}

#[test]
fn update_validators_wrong_owner() {
    let mut deps = mock_dependencies(&[]);

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core1"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![drop_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
                on_top: Some(Uint128::zero()),
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
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![
                drop_staking_base::msg::validatorset::ValidatorData {
                    valoper_address: "valoper_address1".to_string(),
                    weight: 1,
                    on_top: Some(Uint128::new(10)),
                },
                drop_staking_base::msg::validatorset::ValidatorData {
                    valoper_address: "valoper_address2".to_string(),
                    weight: 1,
                    on_top: Some(Uint128::zero()),
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
                on_top: Uint128::new(10),
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
                on_top: Uint128::zero(),
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
fn update_validators_without_ontop_ok() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![
                drop_staking_base::msg::validatorset::ValidatorData {
                    valoper_address: "valoper_address1".to_string(),
                    weight: 1,
                    on_top: Some(Uint128::new(20)),
                },
                drop_staking_base::msg::validatorset::ValidatorData {
                    valoper_address: "valoper_address2".to_string(),
                    weight: 1,
                    on_top: None,
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
                on_top: Uint128::new(20),
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
                on_top: Uint128::zero(),
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
fn update_validators_use_last_ontop() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("core").as_ref()),
    );

    drop_staking_base::state::validatorset::VALIDATORS_SET
        .save(
            deps.as_mut().storage,
            "voter",
            &drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address1".to_string(),
                weight: 0u64,
                on_top: Uint128::new(30),
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0u64,
                total_voted_proposals: 0u64,
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![drop_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address1".to_string(),
                weight: 1,
                on_top: None,
            }],
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
                on_top: Uint128::new(30),
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
        ])
        .unwrap()
    );
}

#[test]
fn update_validators_info_wrong_sender() {
    let mut deps = mock_dependencies(&[]);

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
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    let _response = crate::contract::execute(
        deps_mut,
        mock_env(),
        message_info(&Addr::unchecked("core"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![drop_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
                on_top: Some(Uint128::zero()),
            }],
        },
    )
    .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("stats_contract1"), &[]),
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
fn update_validators_info_ok() {
    let mut deps = mock_dependencies(&[]);

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
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
            validators: vec![drop_staking_base::msg::validatorset::ValidatorData {
                valoper_address: "valoper_address".to_string(),
                weight: 1,
                on_top: Some(Uint128::new(2)),
            }],
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("stats_contract"), &[]),
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
                on_top: Uint128::new(2),
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

#[test]
fn test_execute_update_validators_voting_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("not_provider_proposals_contract"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidatorsVoting {
            proposal: ProposalInfo {
                proposal: neutron_sdk::interchain_queries::v047::types::Proposal {
                    proposal_id: 0u64,
                    proposal_type: None,
                    total_deposit: vec![],
                    status: 0i32,
                    submit_time: None,
                    deposit_end_time: None,
                    voting_start_time: None,
                    voting_end_time: None,
                    final_tally_result: None,
                },
                votes: None,
                is_spam: false,
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::validatorset::ContractError::Unauthorized {}
    );
}

#[test]
fn test_execute_update_validators_voting_spam_proposal() {
    let mut deps = mock_dependencies(&[]);
    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("provider_proposals_contract"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidatorsVoting {
            proposal: ProposalInfo {
                proposal: neutron_sdk::interchain_queries::v047::types::Proposal {
                    proposal_id: 0u64,
                    proposal_type: None,
                    total_deposit: vec![],
                    status: 0i32,
                    submit_time: None,
                    deposit_end_time: None,
                    voting_start_time: None,
                    voting_end_time: None,
                    final_tally_result: None,
                },
                votes: None,
                is_spam: true,
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-staking__drop-validators-set-update_validators_info".to_string()
            )
            .add_attribute("spam_proposal".to_string(), "0".to_string())
        )
    );
}

#[test]
fn test_execute_update_validators_voting() {
    let mut deps = mock_dependencies(&[]);
    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();
    drop_staking_base::state::validatorset::VALIDATORS_SET
        .save(
            deps.as_mut().storage,
            "voter",
            &drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoper_address".to_string(),
                weight: 0u64,
                on_top: Uint128::zero(),
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0u64,
                total_voted_proposals: 0u64,
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("provider_proposals_contract"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidatorsVoting {
            proposal: ProposalInfo {
                proposal: neutron_sdk::interchain_queries::v047::types::Proposal {
                    proposal_id: 0u64,
                    proposal_type: None,
                    total_deposit: vec![],
                    status: 0i32,
                    submit_time: None,
                    deposit_end_time: None,
                    voting_start_time: None,
                    voting_end_time: None,
                    final_tally_result: None,
                },
                votes: Some(vec![
                    neutron_sdk::interchain_queries::v047::types::ProposalVote {
                        proposal_id: 0u64,
                        voter: "voter".to_string(),
                        options: vec![
                            neutron_sdk::interchain_queries::v047::types::WeightedVoteOption {
                                option: 0i32,
                                weight: "weight".to_string(),
                            },
                        ],
                    },
                ]),
                is_spam: false,
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-staking__drop-validators-set-execute_update_validators_voting"
                    .to_string()
            )
            .add_attribute("proposal_id".to_string(), "0".to_string())
        )
    );
}

#[test]
fn query_ownership() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    let response = from_json::<cw_ownable::Ownership<Addr>>(
        &crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::validatorset::QueryMsg::Ownership {},
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        response,
        cw_ownable::Ownership::<Addr> {
            owner: Some(Addr::unchecked("owner")),
            pending_owner: None,
            pending_expiry: None,
        }
    );
}

#[test]
fn execute_update_ownership() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner1")).unwrap();
    }

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("owner1"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::TransferOwnership {
                new_owner: String::from("owner2"),
                expiry: None,
            },
        ),
    )
    .unwrap();
    assert_eq!(response, Response::new());

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("owner2"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::AcceptOwnership,
        ),
    )
    .unwrap();
    assert_eq!(response, Response::new());

    cw_ownable::assert_owner(deps.as_mut().storage, &Addr::unchecked("owner2")).unwrap();
}

#[test]
fn execute_edit_on_top_unauthorized_no_authorizations() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: None,
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("someone"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::EditOnTop { operations: vec![] },
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::validatorset::ContractError::Unauthorized {}
    );
}

#[test]
fn execute_edit_on_top_unauthorized_stranger() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("someone"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::EditOnTop { operations: vec![] },
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::validatorset::ContractError::Unauthorized {}
    );
}

#[test]
fn execute_edit_on_top_authorized_owner() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("owner"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::EditOnTop { operations: vec![] },
    )
    .unwrap();

    assert_eq!(
        response,
        Response::new().add_event(Event::new(
            "crates.io:drop-staking__drop-validators-set-execute-edit-on-top"
        ))
    );
}

#[test]
fn execute_edit_on_top_authorized_val_ref_contract() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("val_ref_contract"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::EditOnTop { operations: vec![] },
    )
    .unwrap();

    assert_eq!(
        response,
        Response::new().add_event(Event::new(
            "crates.io:drop-staking__drop-validators-set-execute-edit-on-top"
        ))
    );
}

#[test]
fn execute_edit_on_top_add() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    drop_staking_base::state::validatorset::VALIDATORS_SET
        .save(
            deps.as_mut().storage,
            "valoperX",
            &drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoperX".to_string(),
                weight: 0u64,
                on_top: Uint128::zero(),
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0u64,
                total_voted_proposals: 0u64,
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("val_ref_contract"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::EditOnTop {
            operations: vec![
                drop_staking_base::msg::validatorset::OnTopEditOperation::Add {
                    validator_address: String::from("valoperX"),
                    amount: Uint128::new(100),
                },
            ],
        },
    )
    .unwrap();

    assert_eq!(
        drop_staking_base::state::validatorset::VALIDATORS_SET
            .load(deps.as_ref().storage, "valoperX")
            .unwrap(),
        drop_staking_base::state::validatorset::ValidatorInfo {
            valoper_address: "valoperX".to_string(),
            weight: 0u64,
            on_top: Uint128::new(100),
            last_processed_remote_height: None,
            last_processed_local_height: None,
            last_validated_height: None,
            last_commission_in_range: None,
            uptime: Decimal::zero(),
            tombstone: false,
            jailed_number: None,
            init_proposal: None,
            total_passed_proposals: 0u64,
            total_voted_proposals: 0u64,
        }
    );

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-validators-set-execute-edit-on-top")
                .add_attribute("valoperX", "100")
        )
    );
}

#[test]
fn execute_edit_on_top_subtract() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    drop_staking_base::state::validatorset::VALIDATORS_SET
        .save(
            deps.as_mut().storage,
            "valoperX",
            &drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoperX".to_string(),
                weight: 0u64,
                on_top: Uint128::new(200),
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0u64,
                total_voted_proposals: 0u64,
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("val_ref_contract"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::EditOnTop {
            operations: vec![
                drop_staking_base::msg::validatorset::OnTopEditOperation::Set {
                    validator_address: String::from("valoperX"),
                    amount: Uint128::new(100),
                },
            ],
        },
    )
    .unwrap();

    assert_eq!(
        drop_staking_base::state::validatorset::VALIDATORS_SET
            .load(deps.as_ref().storage, "valoperX")
            .unwrap(),
        drop_staking_base::state::validatorset::ValidatorInfo {
            valoper_address: "valoperX".to_string(),
            weight: 0u64,
            on_top: Uint128::new(100),
            last_processed_remote_height: None,
            last_processed_local_height: None,
            last_validated_height: None,
            last_commission_in_range: None,
            uptime: Decimal::zero(),
            tombstone: false,
            jailed_number: None,
            init_proposal: None,
            total_passed_proposals: 0u64,
            total_voted_proposals: 0u64,
        }
    );

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-validators-set-execute-edit-on-top")
                .add_attribute("valoperX", "100")
        )
    );
}

#[test]
fn execute_edit_on_top_mixed() {
    let mut deps = mock_dependencies(&[]);

    drop_staking_base::state::validatorset::CONFIG
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::validatorset::Config {
                stats_contract: Addr::unchecked("stats_contract"),
                provider_proposals_contract: Some(Addr::unchecked("provider_proposals_contract")),
                val_ref_contract: Some(Addr::unchecked("val_ref_contract")),
            },
        )
        .unwrap();

    drop_staking_base::state::validatorset::VALIDATORS_SET
        .save(
            deps.as_mut().storage,
            "valoperX",
            &drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoperX".to_string(),
                weight: 0u64,
                on_top: Uint128::new(200),
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0u64,
                total_voted_proposals: 0u64,
            },
        )
        .unwrap();
    drop_staking_base::state::validatorset::VALIDATORS_SET
        .save(
            deps.as_mut().storage,
            "valoperY",
            &drop_staking_base::state::validatorset::ValidatorInfo {
                valoper_address: "valoperY".to_string(),
                weight: 0u64,
                on_top: Uint128::new(0),
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0u64,
                total_voted_proposals: 0u64,
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("val_ref_contract"), &[]),
        drop_staking_base::msg::validatorset::ExecuteMsg::EditOnTop {
            operations: vec![
                drop_staking_base::msg::validatorset::OnTopEditOperation::Set {
                    validator_address: String::from("valoperX"),
                    amount: Uint128::new(100),
                },
                drop_staking_base::msg::validatorset::OnTopEditOperation::Add {
                    validator_address: String::from("valoperY"),
                    amount: Uint128::new(500),
                },
            ],
        },
    )
    .unwrap();

    assert_eq!(
        drop_staking_base::state::validatorset::VALIDATORS_SET
            .load(deps.as_ref().storage, "valoperX")
            .unwrap(),
        drop_staking_base::state::validatorset::ValidatorInfo {
            valoper_address: "valoperX".to_string(),
            weight: 0u64,
            on_top: Uint128::new(100),
            last_processed_remote_height: None,
            last_processed_local_height: None,
            last_validated_height: None,
            last_commission_in_range: None,
            uptime: Decimal::zero(),
            tombstone: false,
            jailed_number: None,
            init_proposal: None,
            total_passed_proposals: 0u64,
            total_voted_proposals: 0u64,
        }
    );

    assert_eq!(
        drop_staking_base::state::validatorset::VALIDATORS_SET
            .load(deps.as_ref().storage, "valoperY")
            .unwrap(),
        drop_staking_base::state::validatorset::ValidatorInfo {
            valoper_address: "valoperY".to_string(),
            weight: 0u64,
            on_top: Uint128::new(500),
            last_processed_remote_height: None,
            last_processed_local_height: None,
            last_validated_height: None,
            last_commission_in_range: None,
            uptime: Decimal::zero(),
            tombstone: false,
            jailed_number: None,
            init_proposal: None,
            total_passed_proposals: 0u64,
            total_voted_proposals: 0u64,
        }
    );

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-validators-set-execute-edit-on-top")
                .add_attributes([("valoperX", "100"), ("valoperY", "500")])
        )
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
        drop_staking_base::msg::validatorset::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::validatorset::ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}
