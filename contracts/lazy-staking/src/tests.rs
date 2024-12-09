use crate::contract::{execute, query};
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::{Config, SplittingTarget, CONFIG, DENOM};
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::BalanceResponse;
use cosmwasm_std::{
    from_json, testing::mock_env, to_json_binary, Api, Coin, Decimal, SupplyResponse, Uint128,
};
use drop_helpers::testing::mock_dependencies_with_api;
use drop_staking_base::state::factory::State;

#[test]
fn test_execute_lazy_denom() {
    let mut deps = mock_dependencies_with_api(&[]);
    let splitting_targets = vec![
        SplittingTarget {
            addr: deps.api.addr_validate("recipient1").unwrap(),
            unbonding_weight: Uint128::from(100u128),
        },
        SplittingTarget {
            addr: deps.api.addr_validate("recipient2").unwrap(),
            unbonding_weight: Uint128::from(100u128),
        },
    ];
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                factory_addr: "factory_contract".to_string(),
                base_denom: "dAsset".to_string(),
                splitting_targets,
            },
        )
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &"lAsset".to_string())
        .unwrap();
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
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
            Uint128::from(200u128),
            Uint128::from(100u128),
        ))
        .unwrap()
    });
    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "sender",
            &[Coin {
                denom: "dAsset".to_string(),
                amount: Uint128::from(100u128),
            }],
        ),
        ExecuteMsg::Bond {},
    )
    .unwrap();
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
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
            Uint128::from(300u128),
            Uint128::from(100u128),
        ))
        .unwrap()
    });
    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "sender",
            &[Coin {
                denom: "dAsset".to_string(),
                amount: Uint128::from(200u128),
            }],
        ),
        ExecuteMsg::Bond {},
    )
    .unwrap();
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
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
            Uint128::from(1000u128),
            Uint128::from(100u128),
        ))
        .unwrap()
    });
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info(
            "sender",
            &[Coin {
                denom: "lAsset".to_string(),
                amount: Uint128::from(800u128),
            }],
        ),
        ExecuteMsg::Unbond {},
    )
    .unwrap();
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
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
            Uint128::from(1000u128),
            Uint128::from(100u128),
        ))
        .unwrap()
    });
    deps.querier.add_bank_query_response(
        mock_env().contract.address.into_string(),
        BalanceResponse {
            amount: Coin {
                denom: "dAsset".to_string(),
                amount: Uint128::from(300u128),
            },
        },
    );
    let mut supply_response = SupplyResponse::default();
    supply_response.amount = Coin::new(800u128, "lAsset".to_string());
    deps.querier
        .add_bank_query_supply_response("lAsset".to_string(), supply_response);
    let exchange_rate: Decimal = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            QueryMsg::ExchangeRate {},
        )
        .unwrap(),
    )
    .unwrap();
    println!("{:?}", res);
    println!("{:?}", exchange_rate);
}
