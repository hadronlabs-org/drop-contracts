use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info},
    Event, Response, StdError,
};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::msg::withdrawal_voucher::ExecuteMsg;
use drop_staking_base::msg::withdrawal_voucher::ExtensionExecuteMsg;
use drop_staking_base::state::withdrawal_voucher::{Pause, PAUSE};

#[test]
fn test_set_pause() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause { mint: false })
        .unwrap();
    let res = crate::contract::entry::execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::Extension {
            msg: ExtensionExecuteMsg::SetPause(Pause { mint: true }),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::default().add_event(
            Event::new("crates.io:drop-staking__drop-withdrawal-voucher-execute-set-pause")
                .add_attributes(vec![attr("mint", "true")])
        )
    );
    assert_eq!(
        PAUSE.load(deps.as_ref().storage).unwrap(),
        Pause { mint: true }
    );
}

#[test]
fn test_mint_unpaused() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause { mint: false })
        .unwrap();
    let res = crate::contract::entry::execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::Mint {
            token_id: "1".to_string(),
            owner: "new_owner".to_string(),
            token_uri: None,
            extension: None,
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::default().add_attributes(vec![
            attr("action", "mint"),
            attr("minter", "owner"),
            attr("owner", "new_owner"),
            attr("token_id", "1")
        ])
    );
}

#[test]
fn test_mint_paused() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    PAUSE
        .save(deps.as_mut().storage, &Pause { mint: true })
        .unwrap();
    let error = crate::contract::entry::execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::Mint {
            token_id: "1".to_string(),
            owner: "new_owner".to_string(),
            token_uri: None,
            extension: None,
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        cw721_base::ContractError::Std(StdError::GenericErr {
            msg: "method mint is paused".to_string(),
        })
    );
}
