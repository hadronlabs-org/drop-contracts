use crate::{
    contract::{execute, instantiate, query},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};
use cosmwasm_std::{
    from_json,
    testing::{mock_env, mock_info},
    Uint64,
};
use drop_helpers::testing::mock_dependencies;
use drop_puppeteer_base::peripheral_hook::ResponseHookMsg as PuppeteerResponseHookMsg;

#[test]
fn save_and_retrieve() {
    let mut deps = mock_dependencies(&[]);

    instantiate(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("creator", &[]),
        InstantiateMsg {},
    )
    .unwrap();

    let total_saved_messages: Uint64 = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            QueryMsg::TotalSavedMessages {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(total_saved_messages, Uint64::zero());

    let transaction = drop_puppeteer_base::peripheral_hook::Transaction::IBCTransfer {
        denom: "remote_denom".to_string(),
        amount: 1000u128,
        real_amount: 1000u128,
        recipient: "recipient".to_string(),
        reason: drop_puppeteer_base::peripheral_hook::IBCTransferReason::Delegate,
    };
    let msg = Box::new(
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(
            drop_puppeteer_base::peripheral_hook::ResponseHookSuccessMsg {
                local_height: 12345,
                remote_height: 54321,
                transaction,
            },
        ),
    );

    execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("puppeteer", &[]),
        ExecuteMsg::PeripheralHook(msg.clone()),
    )
    .unwrap();

    let total_saved_messages: Uint64 = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            QueryMsg::TotalSavedMessages {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(total_saved_messages, Uint64::one());

    let (reporter, saved_message): (String, PuppeteerResponseHookMsg) = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            QueryMsg::SavedMessage { id: Uint64::zero() },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(reporter, "puppeteer");
    assert_eq!(saved_message, *msg);
}
