use crate::{
    contract,
    error::ContractError,
    msg::{BondMsg, ExecuteMsg, InstantiateMsg},
    store::{FACTORY_CONTRACT, LD_TOKEN},
};
use cosmwasm_std::{
    attr, coin,
    testing::{mock_env, mock_info},
    Event,
};
use drop_helpers::testing::{mock_dependencies, mock_locator_query};

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let response = contract::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        InstantiateMsg {
            factory_contract: "factory_contract".to_string(),
            ld_token: "ld_token".to_string(),
        },
    )
    .unwrap();

    let factory_contract = FACTORY_CONTRACT.load(deps.as_ref().storage).unwrap();
    assert_eq!(factory_contract, "factory_contract");
    let ld_token = LD_TOKEN.load(deps.as_ref().storage).unwrap();
    assert_eq!(ld_token, "ld_token");
    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("drop-auto-withdrawer-instantiate").add_attributes([
                attr("factory_contract", "factory_contract"),
                attr("ld_token", "ld_token")
            ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn bond_missing_ld_assets() {
    let mut deps = mock_dependencies(&[]);
    mock_locator_query(&mut deps);
    FACTORY_CONTRACT
        .save(
            deps.as_mut().storage,
            &cosmwasm_std::Addr::unchecked("factory_contract".to_string()),
        )
        .unwrap();
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
    use drop_helpers::testing::{mock_dependencies, mock_locator_query};

    use super::*;

    #[test]
    fn with_ld_assets() {
        let mut deps = mock_dependencies(&[]);
        mock_locator_query(&mut deps);
        FACTORY_CONTRACT
            .save(
                deps.as_mut().storage,
                &cosmwasm_std::Addr::unchecked("factory_contract".to_string()),
            )
            .unwrap();
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
        let mut deps = mock_dependencies(&[]);
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
