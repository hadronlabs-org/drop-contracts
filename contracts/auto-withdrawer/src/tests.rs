use crate::{
    contract,
    error::ContractError,
    msg::{BondMsg, ExecuteMsg, InstantiateMsg},
    store::{CORE_ADDRESS, LD_TOKEN, WITHDRAWAL_MANAGER_ADDRESS, WITHDRAWAL_VOUCHER_ADDRESS},
};
use cosmwasm_std::{
    attr, coin,
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Event, OwnedDeps, Querier,
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
    let response = contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        InstantiateMsg {
            core_address: "core".to_string(),
            withdrawal_voucher_address: "withdrawal_voucher".to_string(),
            withdrawal_manager_address: "withdrawal_manager".to_string(),
            ld_token: "ld_token".to_string(),
        },
    )
    .unwrap();

    let core = CORE_ADDRESS.load(deps.as_ref().storage).unwrap();
    assert_eq!(core, "core");
    let ld_token = LD_TOKEN.load(deps.as_ref().storage).unwrap();
    assert_eq!(ld_token, "ld_token");
    let withdrawal_voucher = WITHDRAWAL_VOUCHER_ADDRESS
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(withdrawal_voucher, "withdrawal_voucher");
    let withdrawal_manager = WITHDRAWAL_MANAGER_ADDRESS
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(withdrawal_manager, "withdrawal_manager");

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("drop-auto-withdrawer-instantiate").add_attributes([
                attr("core_address", "core"),
                attr("withdrawal_voucher", "withdrawal_voucher"),
                attr("withdrawal_manager", "withdrawal_manager"),
                attr("ld_token", "ld_token")
            ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn bond_missing_ld_assets() {
    let mut deps = mock_dependencies::<MockQuerier>();
    LD_TOKEN
        .save(deps.as_mut().storage, &"ld_token".into())
        .unwrap();
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[coin(10, "uatom"), coin(20, "untrn")]),
        ExecuteMsg::Bond(BondMsg::WithLdAssets {}),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::LdTokenExpected {});
}

mod bond_missing_deposit {
    use super::*;

    #[test]
    fn with_ld_assets() {
        let mut deps = mock_dependencies::<MockQuerier>();
        LD_TOKEN
            .save(deps.as_mut().storage, &"ld_token".into())
            .unwrap();
        let err = contract::execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[coin(10, "ld_token")]),
            ExecuteMsg::Bond(BondMsg::WithLdAssets {}),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::DepositExpected {});
    }

    #[test]
    fn with_nft() {
        let mut deps = mock_dependencies::<MockQuerier>();
        let err = contract::execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[]),
            ExecuteMsg::Bond(BondMsg::WithNFT {
                token_id: "token_id".into(),
            }),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::DepositExpected {});
    }
}
