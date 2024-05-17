use std::env;

use crate::{
    contract,
    msg::{FactoryInstance, InstantiateMsg, QueryMsg},
    state::{DropInstance, STATE},
};
use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info, MockQuerier},
    to_json_binary, CosmosMsg,
};
use drop_helpers::testing::{mock_dependencies, WasmMockQuerier};
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

    let expected_factory_state = FactoryState {
        token_contract: String::from("token_contract"),
        core_contract: String::from("core_contract"),
        puppeteer_contract: String::from("puppeteer_contract"),
        staker_contract: String::from("staker_contract"),
        withdrawal_voucher_contract: String::from("withdrawal_voucher_contract"),
        withdrawal_manager_contract: String::from("withdrawal_manager_contract"),
        strategy_contract: String::from("strategy_contract"),
        validators_set_contract: String::from("validators_set_contract"),
        distribution_contract: String::from("distribution_contract"),
        rewards_manager_contract: String::from("rewards_manager_contract"),
    };

    // When we call factory (addr) contract we're expecting to get invalid data as the part of expected behaviour
    deps.querier.add_wasm_query_response("factory", |msg| {
        let q: FactoryQueryMsg = from_json(msg).unwrap();
        match q {
            FactoryQueryMsg::State {} => to_json_binary(&FactoryState {
                token_contract: String::from("token_contract"),
                core_contract: String::from("core_contract"),
                puppeteer_contract: String::from("puppeteer_contract"),
                staker_contract: String::from("staker_contract"),
                withdrawal_voucher_contract: String::from("withdrawal_voucher_contract"),
                withdrawal_manager_contract: String::from("withdrawal_manager_contract"),
                strategy_contract: String::from("strategy_contract"),
                validators_set_contract: String::from("validators_set_contract"),
                distribution_contract: String::from("distribution_contract"),
                rewards_manager_contract: String::from("rewards_manager_contract"),
            })
            .unwrap(),
            _ => unimplemented!(),
        }
    });

    // Drop instance that we'll add and expecting to exist
    let expected_drop_instance = DropInstance {
        name: String::from("chain1"),
        factory_addr: String::from("factory"),
    };

    // Add chain1 with factory as addr of factory instance
    let msg = crate::msg::ExecuteMsg::AddChains {
        chains: vec![expected_drop_instance.clone()],
    };
    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        msg.clone(),
    )
    .unwrap();

    // Get chain1 info (iow trigger proxy query)
    let factory_instance_info1: FactoryInstance = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::FactoryInstance {
                name: String::from("chain1"),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        factory_instance_info1,
        FactoryInstance {
            addr: String::from("factory"),
            contracts: expected_factory_state
        }
    );

    // Get chain1 info (iow trigger STATE)
    let value_load = STATE
        .load(deps.as_ref().storage, "chain1".to_string())
        .unwrap();
    assert_eq!(value_load, expected_drop_instance);

    // Get chain1 info (iow trigger constract's queries)
    let value_query: DropInstance = from_json(
        contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Chain {
                name: String::from("chain1"),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(value_query, expected_drop_instance);
}
