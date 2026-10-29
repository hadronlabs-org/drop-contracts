use crate::{contract, error::ContractError};
use cosmwasm_std::{
    attr, from_json, testing::mock_env, testing::mock_info, to_json_binary, BankMsg, Binary, Coin,
    CosmosMsg, Decimal, Event, Uint128, WasmMsg,
};
use cw721::NftInfoResponse;
use drop_helpers::testing::{mock_dependencies, WasmMockQuerier};
use drop_staking_base::{
    msg::withdrawal_manager::{ExecuteMsg, MigrateMsg, QueryMsg, ReceiveNftMsg},
    state::{
        core::{UnbondBatch, UnbondBatchStatus, UnbondBatchStatusTimestamps},
        withdrawal_manager::{Config, Cw721ReceiveMsg, CONFIG, PAYOUT_RATIO},
        withdrawal_voucher::Metadata,
    },
};
use neutron_sdk::bindings::query::NeutronQuery;
use std::str::FromStr;

type Deps = cosmwasm_std::OwnedDeps<
    cosmwasm_std::testing::MockStorage,
    cosmwasm_std::testing::MockApi,
    WasmMockQuerier,
    NeutronQuery,
>;

fn setup(voucher_amount: u128, total_dasset: u128, unbonded: u128) -> Deps {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                core_contract: cosmwasm_std::Addr::unchecked("core"),
                withdrawal_voucher_contract: cosmwasm_std::Addr::unchecked("voucher"),
                base_denom: "uatom".to_string(),
            },
        )
        .unwrap();
    cw2::set_contract_version(
        deps.as_mut().storage,
        "crates.io:drop-staking__drop-withdrawal-manager",
        "1.0.0",
    )
    .unwrap();
    deps.querier.add_wasm_query_response("voucher", move |_| {
        to_json_binary(&NftInfoResponse {
            token_uri: None,
            extension: Some(Metadata {
                name: "voucher".to_string(),
                description: None,
                attributes: None,
                batch_id: "7".to_string(),
                amount: Uint128::from(voucher_amount),
            }),
        })
        .unwrap()
    });
    deps.querier.add_wasm_query_response("core", move |_| {
        to_json_binary(&UnbondBatch {
            total_dasset_amount_to_withdraw: Uint128::from(total_dasset),
            expected_native_asset_amount: Uint128::from(unbonded),
            expected_release_time: 0,
            total_unbond_items: 2,
            status: UnbondBatchStatus::Withdrawn,
            slashing_effect: Some(Decimal::one()),
            unbonded_amount: Some(Uint128::from(unbonded)),
            withdrawn_amount: None,
            status_timestamps: UnbondBatchStatusTimestamps {
                new: 0,
                unbond_requested: None,
                unbond_failed: None,
                unbonding: None,
                withdrawing: None,
                withdrawn: None,
                withdrawing_emergency: None,
                withdrawn_emergency: None,
            },
        })
        .unwrap()
    });
    deps
}

fn withdraw(deps: &mut Deps) -> cosmwasm_std::Response<neutron_sdk::bindings::msg::NeutronMsg> {
    contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("voucher", &[]),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: "user".to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&ReceiveNftMsg::Withdraw { receiver: None }).unwrap(),
        }),
    )
    .unwrap()
}

fn assert_payout(
    res: &cosmwasm_std::Response<neutron_sdk::bindings::msg::NeutronMsg>,
    amount: u128,
    ratio: &str,
) {
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "user".to_string(),
            amount: vec![Coin::new(amount, "uatom")],
        })
    );
    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "core".to_string(),
            msg: to_json_binary(
                &drop_staking_base::msg::core::ExecuteMsg::UpdateWithdrawnAmount {
                    batch_id: 7,
                    withdrawn_amount: Uint128::from(amount),
                }
            )
            .unwrap(),
            funds: vec![],
        })
    );
    assert_eq!(
        res.events,
        vec![
            Event::new("crates.io:drop-staking__drop-withdrawal-manager-execute-receive_nft")
                .add_attributes(vec![
                    attr("action", "receive_nft"),
                    attr("batch_id", "7"),
                    attr("payout_ratio", ratio),
                    attr("payout_amount", amount.to_string()),
                    attr("to_address", "user"),
                ])
        ]
    );
}

#[test]
fn withdraw_without_ratio_pays_full_share() {
    let mut deps = setup(400, 1000, 1_270_000);
    let res = withdraw(&mut deps);
    assert_payout(&res, 508_000, "1");
}

#[test]
fn migrate_sets_ratio_and_withdraw_applies_haircut() {
    let mut deps = setup(400, 1000, 1_270_000);
    contract::migrate(
        deps.as_mut(),
        mock_env(),
        MigrateMsg {
            payout_ratio: Some(Decimal::from_str("0.7121").unwrap()),
        },
    )
    .unwrap();
    assert_eq!(
        cw2::get_contract_version(deps.as_ref().storage)
            .unwrap()
            .version,
        "1.0.1"
    );

    let ratio: Decimal =
        from_json(contract::query(deps.as_ref(), mock_env(), QueryMsg::PayoutRatio {}).unwrap())
            .unwrap();
    assert_eq!(ratio, Decimal::from_str("0.7121").unwrap());

    // 508_000 * 0.7121 = 361_746.8 -> floored
    let res = withdraw(&mut deps);
    assert_payout(&res, 361_746, "0.7121");
}

#[test]
fn migrate_without_ratio_keeps_stored_value() {
    let mut deps = setup(400, 1000, 1_270_000);
    PAYOUT_RATIO
        .save(deps.as_mut().storage, &Decimal::percent(50))
        .unwrap();
    contract::migrate(deps.as_mut(), mock_env(), MigrateMsg { payout_ratio: None }).unwrap();
    assert_eq!(
        PAYOUT_RATIO.load(deps.as_ref().storage).unwrap(),
        Decimal::percent(50)
    );
}

#[test]
fn migrate_rejects_invalid_ratio() {
    for ratio in [Decimal::zero(), Decimal::percent(101)] {
        let mut deps = setup(400, 1000, 1_270_000);
        let err = contract::migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                payout_ratio: Some(ratio),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidPayoutRatio {});
    }
}

#[test]
fn query_payout_ratio_defaults_to_one() {
    let deps = setup(400, 1000, 1_270_000);
    let ratio: Binary =
        contract::query(deps.as_ref(), mock_env(), QueryMsg::PayoutRatio {}).unwrap();
    assert_eq!(from_json::<Decimal>(&ratio).unwrap(), Decimal::one());
}
