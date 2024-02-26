use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, Event, OwnedDeps, Querier,
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
        lido_staking_base::msg::proposal_votes::InstantiateMsg {
            connection_id: "connection-0".to_string(),
            port_id: "transfer".to_string(),
            update_period: 100,
            core_address: "core".to_string(),
            provider_proposals_address: "provider_proposals".to_string(),
        },
    )
    .unwrap();

    let config = lido_staking_base::state::proposal_votes::CONFIG
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(
        config,
        lido_staking_base::state::proposal_votes::Config {
            connection_id: "connection-0".to_string(),
            port_id: "transfer".to_string(),
            update_period: 100,
            core_address: "core".to_string(),
            provider_proposals_address: "provider_proposals".to_string(),
        }
    );

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:lido-staking__lido-proposal-votes-poc-instantiate")
                .add_attributes([
                    attr("connection_id", "connection-0"),
                    attr("port_id", "transfer"),
                    attr("update_period", "100"),
                    attr("core_address", "core"),
                    attr("provider_proposals_address", "provider_proposals")
                ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies::<MockQuerier>();
    lido_staking_base::state::proposal_votes::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::proposal_votes::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                provider_proposals_address: "provider_proposals".to_string(),
            },
        )
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::proposal_votes::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        response,
        to_json_binary(&lido_staking_base::state::proposal_votes::Config {
            connection_id: "connection-0".to_string(),
            port_id: "transfer".to_string(),
            update_period: 100,
            core_address: "core".to_string(),
            provider_proposals_address: "provider_proposals".to_string()
        })
        .unwrap()
    );
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::proposal_votes::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::proposal_votes::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                provider_proposals_address: "provider_proposals".to_string(),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        lido_staking_base::msg::proposal_votes::ExecuteMsg::UpdateConfig {
            new_config: lido_staking_base::state::proposal_votes::ConfigOptional {
                connection_id: Some("connection-0".to_string()),
                port_id: Some("transfer".to_string()),
                update_period: Some(100),
                core_address: Some("core".to_string()),
                provider_proposals_address: Some("provider_proposals".to_string()),
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

    lido_staking_base::state::proposal_votes::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::proposal_votes::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                provider_proposals_address: "provider_proposals".to_string(),
            },
        )
        .unwrap();

    let _response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        lido_staking_base::msg::proposal_votes::ExecuteMsg::UpdateConfig {
            new_config: lido_staking_base::state::proposal_votes::ConfigOptional {
                connection_id: Some("connection-1".to_string()),
                port_id: Some("transfer1".to_string()),
                update_period: Some(200),
                core_address: Some("core1".to_string()),
                provider_proposals_address: Some("provider_proposals_1".to_string()),
            },
        },
    )
    .unwrap();

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        lido_staking_base::msg::proposal_votes::QueryMsg::Config {},
    )
    .unwrap();

    assert_eq!(
        config,
        to_json_binary(&lido_staking_base::state::proposal_votes::Config {
            connection_id: "connection-1".to_string(),
            port_id: "transfer1".to_string(),
            update_period: 200,
            core_address: "core1".to_string(),
            provider_proposals_address: "provider_proposals_1".to_string()
        })
        .unwrap()
    );
}

#[test]
fn update_voters_list_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core1", &[]),
        lido_staking_base::msg::proposal_votes::ExecuteMsg::UpdateVotersList {
            voters: vec!["voter1".to_string(), "voter2".to_string()],
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
fn update_voters_list_ok() {
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
        lido_staking_base::msg::proposal_votes::ExecuteMsg::UpdateVotersList {
            voters: vec!["voter1".to_string(), "voter2".to_string()],
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    let voters = lido_staking_base::state::proposal_votes::VOTERS
        .load(deps.as_mut().storage)
        .unwrap();

    assert_eq!(voters, vec!["voter1".to_string(), "voter2".to_string()]);
}

#[test]
fn update_active_proposals_wrong_owner() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::proposal_votes::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::proposal_votes::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                provider_proposals_address: "provider_proposals".to_string(),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("wrong_provider_proposals_address", &[]),
        lido_staking_base::msg::proposal_votes::ExecuteMsg::UpdateActiveProposals {
            active_proposals: vec![1],
        },
    )
    .unwrap_err();

    assert_eq!(error, crate::error::ContractError::Unauthorized);
}

#[test]
fn update_active_proposals_ok() {
    let mut deps = mock_dependencies::<MockQuerier>();

    lido_staking_base::state::proposal_votes::CONFIG
        .save(
            deps.as_mut().storage,
            &lido_staking_base::state::proposal_votes::Config {
                connection_id: "connection-0".to_string(),
                port_id: "transfer".to_string(),
                update_period: 100,
                core_address: "core".to_string(),
                provider_proposals_address: "provider_proposals".to_string(),
            },
        )
        .unwrap();

    lido_staking_base::state::proposal_votes::QUERY_ID
        .save(deps.as_mut().storage, &1)
        .unwrap();

    lido_staking_base::state::proposal_votes::VOTERS
        .save(
            deps.as_mut().storage,
            &vec![
                "neutron1x69dz0c0emw8m2c6kp5v6c08kgjxmu30f4a8w5".to_string(),
                "neutron10h9stc5v6ntgeygf5xf945njqq5h32r54rf7kf".to_string(),
            ],
        )
        .unwrap();

    let _response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("provider_proposals", &[]),
        lido_staking_base::msg::proposal_votes::ExecuteMsg::UpdateActiveProposals {
            active_proposals: vec![1, 2],
        },
    )
    .unwrap();

    let active_proposals = lido_staking_base::state::proposal_votes::ACTIVE_PROPOSALS
        .may_load(deps.as_mut().storage)
        .unwrap()
        .unwrap();

    assert_eq!(active_proposals, vec![1, 2]);
}

// TODO: Add more tests
