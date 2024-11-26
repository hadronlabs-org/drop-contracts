use crate::contract::query;
use crate::msg::QueryMsg;
use crate::state::{Config, CONFIG, DENOM};
use cosmwasm_std::BalanceResponse;
use cosmwasm_std::{
    from_json, testing::mock_env, to_json_binary, Api, Coin, Decimal, SupplyResponse, Uint128,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::state::factory::State;

#[test]
fn test_query_exchange_rate() {
    let mut deps = mock_dependencies(&[]);
    let lazy_denom = "lazy_denom".to_string();
    let factory_addr = deps.api.addr_validate("factory_addr").unwrap();
    DENOM.save(deps.as_mut().storage, &lazy_denom).unwrap();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                factory_addr,
                base_denom: "base_denom".to_string(),
                splitting_targets: vec![],
            },
        )
        .unwrap();
    deps.querier.add_wasm_query_response("factory_addr", |_| {
        to_json_binary(&State {
            token_contract: "token_contract".to_string(),
            core_contract: "core_contract".to_string(),
            puppeteer_contract: "puppeteer_contract".to_string(),
            withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
            withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
            strategy_contract: "strategy_contract".to_string(),
            validators_set_contract: "validators_set_contract".to_string(),
            distribution_contract: "distribution_contract".to_string(),
            rewards_manager_contract: "rewards_manager_contract".to_string(),
            rewards_pump_contract: "rewards_pump_contract".to_string(),
            splitter_contract: "splitter_contract".to_string(),
            lsm_share_bond_provider_contract: "lsm_share_bond_provider_contract".to_string(),
            native_bond_provider_contract: "native_bond_provider_contract".to_string(),
        })
        .unwrap()
    });
    deps.querier.add_wasm_query_response("core_contract", |_| {
        to_json_binary(&Decimal::from_ratio(
            Uint128::from(123u128),
            Uint128::from(100u128),
        ))
        .unwrap()
    });
    let mut supply_response = SupplyResponse::default();
    supply_response.amount = Coin {
        denom: lazy_denom.clone(),
        amount: Uint128::from(100u128),
    };
    deps.querier
        .add_bank_query_supply_response(lazy_denom.clone(), supply_response);
    deps.querier.add_bank_query_response(
        mock_env().contract.address.to_string(),
        BalanceResponse {
            amount: Coin {
                denom: "base_denom".to_string(),
                amount: Uint128::from(100u128),
            },
        },
    );
    let res: Decimal = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            QueryMsg::ExchangeRate {},
        )
        .unwrap(),
    )
    .unwrap();
    println!("{:?}", res)
}
