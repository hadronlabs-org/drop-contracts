use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, Binary, Decimal, Event, OwnedDeps, Querier, SubMsg,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery, types::KVKey};
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
        lido_staking_base::msg::provider_proposals::InstantiateMsg {
            connection_id: "connection-0".to_string(),
            port_id: "transfer".to_string(),
            update_period: 100,
            core_address: "core".to_string(),
            validators_set_address: "validators_set".to_string(),
            init_proposal: 1,
            proposals_prefetch: 5,
            veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
        },
    )
    .unwrap();

    let config = lido_staking_base::state::provider_proposals::CONFIG
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(
        config,
        lido_staking_base::state::provider_proposals::Config {
            connection_id: "connection-0".to_string(),
            port_id: "transfer".to_string(),
            update_period: 100,
            core_address: "core".to_string(),
            proposal_votes_address: None,
            validators_set_address: "validators_set".to_string(),
            init_proposal: 1,
            proposals_prefetch: 5,
            veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
        }
    );

    assert_eq!(response.messages.len(), 1);
    let sub_msg = SubMsg::reply_on_success(
        NeutronMsg::RegisterInterchainQuery {
            query_type: "kv".to_string(),
            keys: vec![
                KVKey {
                    path: "gov".to_string(),
                    key: Binary(vec![0, 0, 0, 0, 0, 0, 0, 0, 1]),
                },
                KVKey {
                    path: "gov".to_string(),
                    key: Binary(vec![0, 0, 0, 0, 0, 0, 0, 0, 2]),
                },
                KVKey {
                    path: "gov".to_string(),
                    key: Binary(vec![0, 0, 0, 0, 0, 0, 0, 0, 3]),
                },
                KVKey {
                    path: "gov".to_string(),
                    key: Binary(vec![0, 0, 0, 0, 0, 0, 0, 0, 4]),
                },
                KVKey {
                    path: "gov".to_string(),
                    key: Binary(vec![0, 0, 0, 0, 0, 0, 0, 0, 5]),
                },
            ],
            transactions_filter: "".to_string(),
            connection_id: "connection-0".to_string(),
            update_period: 100,
        },
        1,
    );
    assert_eq!(response.messages, vec![sub_msg]);

    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:lido-staking__lido-provider-proposals-instantiate")
                .add_attributes([
                    attr("connection_id", "connection-0"),
                    attr("port_id", "transfer"),
                    attr("update_period", "100"),
                    attr("core_address", "core"),
                    attr("validators_set_address", "validators_set"),
                    attr("init_proposal", "1"),
                    attr("proposals_prefetch", "5"),
                    attr("veto_spam_threshold", "0.01")
                ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps: OwnedDeps<cosmwasm_std::MemoryStorage, MockApi, MockQuerier, NeutronQuery> =
        mock_dependencies::<MockQuerier>();
    lido_staking_base::state::provider_proposals::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::provider_proposals::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                proposal_votes_address: Some("proposal_votes".to_string()),
                validators_set_address: "validators_set".to_string(),
                init_proposal: 1,
                proposals_prefetch: 5,
                veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
            },
        )
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::provider_proposals::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        response,
        to_json_binary(&lido_staking_base::state::provider_proposals::Config {
            connection_id: "connection-0".to_string(),
            port_id: "transfer".to_string(),
            update_period: 100,
            core_address: "core".to_string(),
            proposal_votes_address: Some("proposal_votes".to_string()),
            validators_set_address: "validators_set".to_string(),
            init_proposal: 1,
            proposals_prefetch: 5,
            veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
        })
        .unwrap()
    );
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::provider_proposals::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::provider_proposals::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                proposal_votes_address: Some("proposal_votes".to_string()),
                validators_set_address: "validators_set".to_string(),
                init_proposal: 1,
                proposals_prefetch: 5,
                veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        lido_staking_base::msg::provider_proposals::ExecuteMsg::UpdateConfig {
            new_config: lido_staking_base::state::provider_proposals::ConfigOptional {
                connection_id: Some("connection-0".to_string()),
                port_id: Some("transfer".to_string()),
                update_period: Some(100),
                core_address: Some("core".to_string()),
                proposal_votes_address: Some("proposal_votes".to_string()),
                validators_set_address: Some("validators_set".to_string()),
                init_proposal: None,
                proposals_prefetch: Some(5),
                veto_spam_threshold: Some(Decimal::from_atomics(1u64, 2).unwrap()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        crate::error::ContractError::OwnershipError(cw_ownable::OwnershipError::Std(
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

    lido_staking_base::state::provider_proposals::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::provider_proposals::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                proposal_votes_address: Some("proposal_votes".to_string()),
                validators_set_address: "validators_set".to_string(),
                init_proposal: 1,
                proposals_prefetch: 5,
                veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        lido_staking_base::msg::provider_proposals::ExecuteMsg::UpdateConfig {
            new_config: lido_staking_base::state::provider_proposals::ConfigOptional {
                connection_id: Some("connection-1".to_string()),
                port_id: Some("transfer1".to_string()),
                update_period: Some(200),
                core_address: Some("core1".to_string()),
                proposal_votes_address: Some("proposal_votes_1".to_string()),
                validators_set_address: Some("validators_set_1".to_string()),
                proposals_prefetch: Some(7),
                init_proposal: None,
                veto_spam_threshold: Some(Decimal::from_atomics(3u64, 2).unwrap()),
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::provider_proposals::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        config,
        to_json_binary(&lido_staking_base::state::provider_proposals::Config {
            connection_id: "connection-1".to_string(),
            port_id: "transfer1".to_string(),
            update_period: 200,
            core_address: "core1".to_string(),
            proposal_votes_address: Some("proposal_votes_1".to_string()),
            validators_set_address: "validators_set_1".to_string(),
            init_proposal: 1,
            proposals_prefetch: 7,
            veto_spam_threshold: Decimal::from_atomics(3u64, 2).unwrap(),
        })
        .unwrap()
    );
}

#[test]
fn update_votes_wrong_sender_address() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::provider_proposals::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::provider_proposals::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                proposal_votes_address: Some("proposal_votes".to_string()),
                validators_set_address: "validators_set".to_string(),
                init_proposal: 1,
                proposals_prefetch: 5,
                veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("proposal_votes_1", &[]),
        lido_staking_base::msg::provider_proposals::ExecuteMsg::UpdateProposalVotes {
            votes: vec![neutron_sdk::interchain_queries::v045::types::ProposalVote {
                proposal_id: 1,
                voter: "voter".to_string(),
                options: vec![],
            }],
        },
    )
    .unwrap_err();

    assert_eq!(error, crate::error::ContractError::Unauthorized);
}

#[test]
fn update_votes_ok() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::provider_proposals::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::provider_proposals::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                proposal_votes_address: Some("proposal_votes".to_string()),
                validators_set_address: "validators_set".to_string(),
                init_proposal: 1,
                proposals_prefetch: 5,
                veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("proposal_votes", &[]),
        lido_staking_base::msg::provider_proposals::ExecuteMsg::UpdateProposalVotes {
            votes: vec![neutron_sdk::interchain_queries::v045::types::ProposalVote {
                proposal_id: 1,
                voter: "voter".to_string(),
                options: vec![],
            }],
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let validator = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::provider_proposals::QueryMsg::GetProposals {},
    )
    .unwrap();

    assert_eq!(
        validator,
        to_json_binary(&Vec::<
            lido_staking_base::state::provider_proposals::ProposalInfo,
        >::new())
        .unwrap()
    );
}

#[test]
fn update_votes_with_data() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::provider_proposals::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::provider_proposals::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                proposal_votes_address: Some("proposal_votes".to_string()),
                validators_set_address: "validators_set".to_string(),
                init_proposal: 1,
                proposals_prefetch: 5,
                veto_spam_threshold: Decimal::from_atomics(1u64, 2).unwrap(),
            },
        )
        .unwrap();

    lido_staking_base::state::provider_proposals::PROPOSALS
        .save(
            deps.as_mut().storage,
            1u64,
            &neutron_sdk::interchain_queries::v045::types::Proposal {
                proposal_id: 1,
                proposal_type: Some("proposal_type".to_string()),
                total_deposit: vec![],
                status: 1,
                submit_time: None,
                deposit_end_time: None,
                voting_start_time: None,
                voting_end_time: None,
                final_tally_result: None,
            },
        )
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("proposal_votes", &[]),
        lido_staking_base::msg::provider_proposals::ExecuteMsg::UpdateProposalVotes {
            votes: vec![neutron_sdk::interchain_queries::v045::types::ProposalVote {
                proposal_id: 1,
                voter: "voter".to_string(),
                options: vec![
                    neutron_sdk::interchain_queries::v045::types::WeightedVoteOption {
                        option: 1,
                        weight: "100".to_string(),
                    },
                ],
            }],
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let validator = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::provider_proposals::QueryMsg::GetProposals {},
    )
    .unwrap();

    assert_eq!(
        validator,
        to_json_binary(&vec![
            lido_staking_base::state::provider_proposals::ProposalInfo {
                proposal: neutron_sdk::interchain_queries::v045::types::Proposal {
                    proposal_id: 1,
                    proposal_type: Some("proposal_type".to_string()),
                    total_deposit: vec![],
                    status: 1,
                    submit_time: None,
                    deposit_end_time: None,
                    voting_start_time: None,
                    voting_end_time: None,
                    final_tally_result: None,
                },
                votes: Some(vec![
                    neutron_sdk::interchain_queries::v045::types::ProposalVote {
                        proposal_id: 1,
                        voter: "voter".to_string(),
                        options: vec![
                            neutron_sdk::interchain_queries::v045::types::WeightedVoteOption {
                                option: 1,
                                weight: "100".to_string(),
                            }
                        ],
                    }
                ]),
                is_spam: false,
            }
        ])
        .unwrap()
    );
}

// TODO: Add more tests
