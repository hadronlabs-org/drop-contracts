use crate::{
    contract::{self, EDIT_ON_TOP_REPLY_ID},
    error::ContractError,
};
use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Decimal, Event, Order, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::{
    msg::{
        core::{BondHook, QueryMsg as CoreQueryMsg},
        val_ref::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, Ref},
        validatorset::{ExecuteMsg as ValidatorSetExecuteMsg, OnTopEditOperation},
    },
    state::val_ref::{CORE_ADDRESS, REFS, VALIDATORS_SET_ADDRESS},
};

fn get_bond_hook_msg(amount: u128, r#ref: Option<&str>) -> BondHook {
    BondHook {
        dasset_minted: amount.into(),
        r#ref: r#ref.map(|r#ref| r#ref.into()),
        amount: Uint128::zero(),     // never used by contract
        denom: String::from(""),     // never used by contract
        sender: Addr::unchecked(""), // never used by contract
    }
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        InstantiateMsg {
            owner: String::from("owner"),
            core_address: String::from("core"),
            validators_set_address: String::from("validators_set"),
        },
    )
    .unwrap();

    cw_ownable::assert_owner(deps.as_ref().storage, &Addr::unchecked("owner")).unwrap();

    assert_eq!(
        CORE_ADDRESS.load(deps.as_ref().storage).unwrap(),
        Addr::unchecked("core")
    );

    assert_eq!(
        VALIDATORS_SET_ADDRESS.load(deps.as_ref().storage).unwrap(),
        Addr::unchecked("validators_set")
    );

    assert_eq!(
        response,
        Response::new().add_event(Event::new("drop-val-ref-instantiate").add_attributes([
            ("core_address", "core"),
            ("validators_set_address", "validators_set")
        ]))
    );
}

#[test]
fn execute_update_ownership() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner1")).unwrap();
    }

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
            new_owner: String::from("owner2"),
            expiry: None,
        }),
    )
    .unwrap();
    assert_eq!(
        response,
        Response::new().add_event(Event::new("drop-val-ref-execute-update-ownership"))
    );

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner2", &[]),
        ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership),
    )
    .unwrap();
    assert_eq!(
        response,
        Response::new().add_event(Event::new("drop-val-ref-execute-update-ownership"))
    );

    cw_ownable::assert_owner(deps.as_mut().storage, &Addr::unchecked("owner2")).unwrap();
}

#[test]
fn execute_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    let error = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stranger", &[]),
        ExecuteMsg::UpdateConfig {
            core_address: String::from("core"),
            validators_set_address: String::from("validators_set"),
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
    );
}

#[test]
fn execute_update_config() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            core_address: String::from("core"),
            validators_set_address: String::from("validators_set"),
        },
    )
    .unwrap();

    assert_eq!(
        CORE_ADDRESS.load(deps.as_ref().storage).unwrap(),
        Addr::unchecked("core")
    );
    assert_eq!(
        VALIDATORS_SET_ADDRESS.load(deps.as_ref().storage).unwrap(),
        Addr::unchecked("validators_set")
    );

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("drop-val-ref-execute-update-config").add_attributes([
                ("core_address", "core"),
                ("validators_set_address", "validators_set")
            ])
        )
    );
}

#[test]
fn execute_bond_hook_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();

    let error = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stranger", &[]),
        ExecuteMsg::BondCallback(get_bond_hook_msg(0, None)),
    )
    .unwrap_err();

    assert_eq!(error, ContractError::Unauthorized {});
}

#[test]
fn execute_bond_hook_no_ref() {
    let mut deps = mock_dependencies(&[]);

    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        ExecuteMsg::BondCallback(get_bond_hook_msg(0, None)),
    )
    .unwrap();

    assert_eq!(
        response,
        Response::new().add_event(Event::new("drop-val-ref-execute-bond-hook"))
    );
}

#[test]
fn execute_bond_hook_unknown_validator() {
    let mut deps = mock_dependencies(&[]);

    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        ExecuteMsg::BondCallback(get_bond_hook_msg(0, Some("X"))),
    )
    .unwrap();

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("drop-val-ref-execute-bond-hook")
                .add_attributes([("ref", "X"), ("validator", "None")])
        )
    );
}

#[test]
fn execute_bond_hook_known_validator() {
    let mut deps = mock_dependencies(&[]);

    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    VALIDATORS_SET_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("validators_set"))
        .unwrap();
    REFS.save(deps.as_mut().storage, "X", &String::from("valoperX"))
        .unwrap();

    deps.querier.add_wasm_query_response("core", |req| {
        let req = from_json::<CoreQueryMsg>(req).unwrap();
        assert_eq!(req, CoreQueryMsg::ExchangeRate {});

        to_json_binary(&Decimal::from_ratio(3u128, 2u128)).unwrap()
    });

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("core", &[]),
        ExecuteMsg::BondCallback(get_bond_hook_msg(100, Some("X"))),
    )
    .unwrap();

    assert_eq!(
        response,
        Response::new()
            .add_submessage(SubMsg::reply_on_error(
                WasmMsg::Execute {
                    contract_addr: String::from("validators_set"),
                    msg: to_json_binary(&ValidatorSetExecuteMsg::EditOnTop {
                        operations: vec![OnTopEditOperation::Add {
                            validator_address: String::from("valoperX"),
                            amount: Uint128::new(150),
                        }]
                    })
                    .unwrap(),
                    funds: vec![]
                },
                EDIT_ON_TOP_REPLY_ID
            ))
            .add_event(
                Event::new("drop-val-ref-execute-bond-hook").add_attributes([
                    ("ref", "X"),
                    ("validator", "valoperX"),
                    ("on_top_increase", "150")
                ])
            )
    );
}

#[test]
fn execute_set_refs_unauthorized() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    let error = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("stranger", &[]),
        ExecuteMsg::SetRefs { refs: vec![] },
    )
    .unwrap_err();

    assert_eq!(
        error,
        ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
    );
}

#[test]
fn execute_set_refs_empty() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    REFS.save(deps.as_mut().storage, "x", &String::from("X"))
        .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::SetRefs { refs: vec![] },
    )
    .unwrap();

    assert_eq!(
        REFS.keys(deps.as_ref().storage, None, None, Order::Ascending)
            .count(),
        0
    );

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("drop-val-ref-execute-set-refs").add_attribute("total_refs", "0")
        )
    );
}

#[test]
fn execute_set_refs_override() {
    let mut deps = mock_dependencies(&[]);

    {
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    }

    REFS.save(deps.as_mut().storage, "x", &String::from("X"))
        .unwrap();

    let response = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::SetRefs {
            refs: vec![
                Ref {
                    r#ref: String::from("y"),
                    validator_address: String::from("valoperY"),
                },
                Ref {
                    r#ref: String::from("z"),
                    validator_address: String::from("valoperZ"),
                },
            ],
        },
    )
    .unwrap();

    assert_eq!(
        REFS.range(deps.as_ref().storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap(),
        vec![
            (String::from("y"), String::from("valoperY")),
            (String::from("z"), String::from("valoperZ"))
        ]
    );

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("drop-val-ref-execute-set-refs").add_attribute("total_refs", "2")
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
        &contract::query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap(),
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
fn query_config() {
    let mut deps = mock_dependencies(&[]);

    CORE_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("core"))
        .unwrap();
    VALIDATORS_SET_ADDRESS
        .save(deps.as_mut().storage, &Addr::unchecked("validators_set"))
        .unwrap();

    let response = from_json::<ConfigResponse>(
        &contract::query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap(),
    )
    .unwrap();

    assert_eq!(
        response,
        ConfigResponse {
            core_address: String::from("core"),
            validators_set_address: String::from("validators_set"),
        }
    );
}

#[test]
fn query_ref() {
    let mut deps = mock_dependencies(&[]);

    REFS.save(deps.as_mut().storage, "x", &String::from("valoperX"))
        .unwrap();

    let response = from_json::<Ref>(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Ref {
                r#ref: String::from("x"),
            },
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        response,
        Ref {
            r#ref: String::from("x"),
            validator_address: String::from("valoperX"),
        }
    )
}

#[test]
fn query_all_refs() {
    let mut deps = mock_dependencies(&[]);

    REFS.save(deps.as_mut().storage, "x", &String::from("valoperX"))
        .unwrap();
    REFS.save(deps.as_mut().storage, "y", &String::from("valoperY"))
        .unwrap();

    let response = from_json::<Vec<Ref>>(
        &contract::query(deps.as_ref(), mock_env(), QueryMsg::AllRefs {}).unwrap(),
    )
    .unwrap();

    assert_eq!(
        response,
        vec![
            Ref {
                r#ref: String::from("x"),
                validator_address: String::from("valoperX"),
            },
            Ref {
                r#ref: String::from("y"),
                validator_address: String::from("valoperY"),
            }
        ]
    )
}

#[test]
fn test_migrate_wrong_contract() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    cw2::set_contract_version(deps_mut.storage, "wrong_contract_name", "0.0.1").unwrap();

    let res = crate::contract::migrate(
        deps.as_mut().into_empty(),
        mock_env(),
        drop_staking_base::msg::val_ref::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: contract::CONTRACT_NAME.to_string()
        }
    )
}
