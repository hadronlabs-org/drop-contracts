use crate::{
    contract,
    msg::{FactoryInstance, InstantiateMsg, QueryMsg},
    state::{DropInstance, STATE},
};
use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info},
    to_json_binary,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::msg::factory::QueryMsg as FactoryQueryMsg;
use drop_staking_base::state::factory::State as FactoryState;

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        InstantiateMsg {},
    )
    .unwrap();
}

#[test]
fn add_factory_instances() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("admin")).unwrap(); // to update admin for contract

    let expected_factory_states = vec![
        FactoryState {
            token_contract: String::from("token_contract1"),
            core_contract: String::from("core_contract1"),
            puppeteer_contract: String::from("puppeteer_contract1"),
            staker_contract: String::from("staker_contract1"),
            withdrawal_voucher_contract: String::from("withdrawal_voucher_contract1"),
            withdrawal_manager_contract: String::from("withdrawal_manager_contract1"),
            strategy_contract: String::from("strategy_contract1"),
            validators_set_contract: String::from("validators_set_contract1"),
            distribution_contract: String::from("distribution_contract1"),
            rewards_manager_contract: String::from("rewards_manager_contract1"),
        },
        FactoryState {
            token_contract: String::from("token_contract2"),
            core_contract: String::from("core_contract2"),
            puppeteer_contract: String::from("puppeteer_contract2"),
            staker_contract: String::from("staker_contract2"),
            withdrawal_voucher_contract: String::from("withdrawal_voucher_contract2"),
            withdrawal_manager_contract: String::from("withdrawal_manager_contract2"),
            strategy_contract: String::from("strategy_contract2"),
            validators_set_contract: String::from("validators_set_contract2"),
            distribution_contract: String::from("distribution_contract2"),
            rewards_manager_contract: String::from("rewards_manager_contract2"),
        },
    ];

    let factory_state_1 = expected_factory_states[0].clone();
    let factory_state_2 = expected_factory_states[1].clone();

    // When we call factory (addr) contract we're expecting to get invalid data as the part of expected behaviour
    deps.querier
        .add_wasm_query_response("factory1", move |msg| {
            let q: FactoryQueryMsg = from_json(msg).unwrap();
            match q {
                FactoryQueryMsg::State {} => to_json_binary(&factory_state_1).unwrap(),
                _ => unimplemented!(),
            }
        });
    deps.querier
        .add_wasm_query_response("factory2", move |msg| {
            let q: FactoryQueryMsg = from_json(msg).unwrap();
            match q {
                FactoryQueryMsg::State {} => to_json_binary(&factory_state_2).unwrap(),
                _ => unimplemented!(),
            }
        });

    // Drop instance that we'll add and expecting to exist
    let expected_drop_instances = vec![
        DropInstance {
            name: String::from("chain1"),
            factory_addr: String::from("factory1"),
        },
        DropInstance {
            name: String::from("chain2"),
            factory_addr: String::from("factory2"),
        },
    ];

    // Add chain1 with factory as addr of factory instance
    let msg = crate::msg::ExecuteMsg::AddChains {
        chains: expected_drop_instances.clone(),
    };
    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        msg.clone(),
    )
    .unwrap();

    // Query each drop_instance we have and compare it with expacted
    for (i, edi) in expected_drop_instances.iter().enumerate() {
        let factory_instance_info: FactoryInstance = from_json(
            crate::contract::query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::FactoryInstance {
                    name: edi.name.clone(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            factory_instance_info,
            FactoryInstance {
                addr: edi.factory_addr.clone(),
                contracts: expected_factory_states[i].clone()
            }
        );
    }

    // Get all possible chains from contract directly from STATE
    for drop_instance in expected_drop_instances.clone() {
        let value_load = STATE
            .load(deps.as_ref().storage, drop_instance.name.to_string())
            .unwrap();
        assert_eq!(value_load, drop_instance);
    }

    // Get all possible chains from contract from query
    for drop_instance in expected_drop_instances {
        let value_query: DropInstance = from_json(
            contract::query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Chain {
                    name: drop_instance.name.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(value_query, drop_instance);
    }
}

#[test]
fn remove_factory_instances() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("admin")).unwrap(); // to update admin for contract

    let expected_factory_states = vec![
        FactoryState {
            token_contract: String::from("token_contract1"),
            core_contract: String::from("core_contract1"),
            puppeteer_contract: String::from("puppeteer_contract1"),
            staker_contract: String::from("staker_contract1"),
            withdrawal_voucher_contract: String::from("withdrawal_voucher_contract1"),
            withdrawal_manager_contract: String::from("withdrawal_manager_contract1"),
            strategy_contract: String::from("strategy_contract1"),
            validators_set_contract: String::from("validators_set_contract1"),
            distribution_contract: String::from("distribution_contract1"),
            rewards_manager_contract: String::from("rewards_manager_contract1"),
        },
        FactoryState {
            token_contract: String::from("token_contract2"),
            core_contract: String::from("core_contract2"),
            puppeteer_contract: String::from("puppeteer_contract2"),
            staker_contract: String::from("staker_contract2"),
            withdrawal_voucher_contract: String::from("withdrawal_voucher_contract2"),
            withdrawal_manager_contract: String::from("withdrawal_manager_contract2"),
            strategy_contract: String::from("strategy_contract2"),
            validators_set_contract: String::from("validators_set_contract2"),
            distribution_contract: String::from("distribution_contract2"),
            rewards_manager_contract: String::from("rewards_manager_contract2"),
        },
    ];

    let factory_state_1 = expected_factory_states[0].clone();
    let factory_state_2 = expected_factory_states[1].clone();

    deps.querier
        .add_wasm_query_response("factory1", move |msg| {
            let q: FactoryQueryMsg = from_json(msg).unwrap();
            match q {
                FactoryQueryMsg::State {} => to_json_binary(&factory_state_1).unwrap(),
                _ => unimplemented!(),
            }
        });
    deps.querier
        .add_wasm_query_response("factory2", move |msg| {
            let q: FactoryQueryMsg = from_json(msg).unwrap();
            match q {
                FactoryQueryMsg::State {} => to_json_binary(&factory_state_2).unwrap(),
                _ => unimplemented!(),
            }
        });

    // Drop instance that we'll add and expecting to exist
    let expected_drop_instances = vec![
        DropInstance {
            name: String::from("chain1"),
            factory_addr: String::from("factory1"),
        },
        DropInstance {
            name: String::from("chain2"),
            factory_addr: String::from("factory2"),
        },
    ];

    // Add chain1 with factory as addr of factory instance
    let msg = crate::msg::ExecuteMsg::AddChains {
        chains: expected_drop_instances.clone(),
    };
    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        msg.clone(),
    )
    .unwrap();

    let drop_instances = vec![
        DropInstance {
            name: String::from("chain1"),
            factory_addr: String::from("factory1"),
        },
        DropInstance {
            name: String::from("chain2"),
            factory_addr: String::from("factory2"),
        },
    ];

    // Check there is no instances in current contract
    for di in drop_instances {
        // Check Drop instance exist
        let drop_instance: DropInstance = from_json(
            crate::contract::query(
                deps.as_ref(),
                mock_env(),
                crate::msg::QueryMsg::Chain {
                    name: di.name.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(drop_instance, di);

        // Remove factory instances from contract's STATE
        let msg = crate::msg::ExecuteMsg::RemoveChains {
            names: vec![di.name.to_string()],
        };
        crate::contract::execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            msg.clone(),
        )
        .unwrap();

        // Check Drop instance doesn't exist
        let drop_instance_new = crate::contract::query(
            deps.as_ref(),
            mock_env(),
            crate::msg::QueryMsg::Chain {
                name: di.name.to_string(),
            },
        );

        match drop_instance_new {
            Ok(_) => panic!("There shouldn't be anything!"),
            Err(..) => (),
        }
    }

    // Check there is no data in contract's STATE
    let available_chains: Vec<DropInstance> = from_json(
        crate::contract::query(deps.as_ref(), mock_env(), crate::msg::QueryMsg::Chains {}).unwrap(),
    )
    .unwrap();

    assert_eq!(available_chains.len(), 0);
}
