use cosmwasm_std::{
    attr, from_json, testing::mock_env, Addr, BankMsg, Coin, CosmosMsg, Event, Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::splitter::Config;

#[test]
fn change_splitter_config() {
    let mut deps = mock_dependencies(&[]);
    let instantiate_config: Config = Config {
        receivers: vec![("receiver1".to_string(), Uint128::from(1000000000u64))],
        denom: "drop".to_string(),
    };
    {
        let _ = crate::contract::instantiate(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::MessageInfo {
                sender: Addr::unchecked("arbitrary_owner"),
                funds: vec![],
            },
            drop_staking_base::msg::splitter::InstantiateMsg {
                config: instantiate_config.clone(),
            },
        );
        let response: cw_ownable::Ownership<String> = from_json(
            crate::contract::query(
                deps.as_ref().into_empty(),
                mock_env(),
                drop_staking_base::msg::splitter::QueryMsg::Ownership {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(response.owner.unwrap(), "arbitrary_owner");
    }
    {
        let response: Config = from_json(
            crate::contract::query(
                deps.as_ref().into_empty(),
                mock_env(),
                drop_staking_base::msg::splitter::QueryMsg::Config {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(response, instantiate_config);
    }
    {
        let new_config: Config = Config {
            receivers: vec![
                ("receiver1".to_string(), Uint128::from(1u64)),
                ("receiver2".to_string(), Uint128::from(2u64)),
                ("receiver3".to_string(), Uint128::from(3u64)),
                ("receiver4".to_string(), Uint128::from(4u64)),
            ],
            denom: "drop".to_string(),
        };
        let _ = crate::contract::execute(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::MessageInfo {
                sender: Addr::unchecked("arbitrary_owner"),
                funds: vec![],
            },
            drop_staking_base::msg::splitter::ExecuteMsg::UpdateConfig {
                new_config: new_config.clone(),
            },
        );
        let response: Config = from_json(
            crate::contract::query(
                deps.as_ref().into_empty(),
                mock_env(),
                drop_staking_base::msg::splitter::QueryMsg::Config {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(response, new_config);
    }
}

#[test]
fn splitter_distribute() {
    let mut deps = mock_dependencies(&[Coin::new(10u128, "drop")]);
    let instantiate_config: Config = Config {
        receivers: vec![
            ("receiver1".to_string(), Uint128::from(1u64)),
            ("receiver2".to_string(), Uint128::from(2u64)),
            ("receiver3".to_string(), Uint128::from(3u64)),
            ("receiver4".to_string(), Uint128::from(4u64)),
        ],
        denom: "drop".to_string(),
    };
    {
        let _ = crate::contract::instantiate(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::MessageInfo {
                sender: Addr::unchecked("arbitrary_owner"),
                funds: vec![],
            },
            drop_staking_base::msg::splitter::InstantiateMsg {
                config: instantiate_config.clone(),
            },
        );
    }
    {
        let response = crate::contract::execute(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::MessageInfo {
                sender: Addr::unchecked("arbitrary_owner"),
                funds: vec![],
            },
            drop_staking_base::msg::splitter::ExecuteMsg::Distribute {},
        )
        .unwrap();
        assert_eq!(
            response,
            cosmwasm_std::Response::new()
                .add_event(
                    Event::new("crates.io:drop-staking__drop-splitter-execute-distribute")
                        .add_attributes(vec![
                            attr("total_shares", "10"),
                            attr("receiver1", "1"),
                            attr("receiver2", "2"),
                            attr("receiver3", "3"),
                            attr("receiver4", "4"),
                        ])
                )
                .add_submessages(vec![
                    cosmwasm_std::SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                        to_address: "receiver1".to_string(),
                        amount: vec![Coin::new(1u128, "drop")]
                    })),
                    cosmwasm_std::SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                        to_address: "receiver2".to_string(),
                        amount: vec![Coin::new(2u128, "drop")]
                    })),
                    cosmwasm_std::SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                        to_address: "receiver3".to_string(),
                        amount: vec![Coin::new(3u128, "drop")]
                    })),
                    cosmwasm_std::SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                        to_address: "receiver4".to_string(),
                        amount: vec![Coin::new(4u128, "drop")]
                    }))
                ])
        );
    }
}

#[test]
fn test_migrate_wrong_contract() {
    let mut deps = mock_dependencies(&[]);

    let deps_mut = deps.as_mut();

    cw2::set_contract_version(deps_mut.storage, "wrong_contract_name", "0.0.1").unwrap();

    let res = crate::contract::migrate(
        deps_mut.into_empty(),
        mock_env(),
        drop_staking_base::msg::splitter::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::splitter::ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}
