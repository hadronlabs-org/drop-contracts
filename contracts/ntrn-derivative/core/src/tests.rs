use crate::{
    contract::{execute, instantiate, query},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{DENOM, SALT},
};
use cosmwasm_std::{
    attr, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, BankMsg, Binary, CosmosMsg, DenomMetadata, Event, ReplyOn, Response, SubMsg,
    WasmMsg,
};
use drop_helpers::testing::{mock_dependencies, mock_dependencies_with_api};
use drop_staking_base::msg::ntrn_derivative::withdrawal_voucher::{
    ExecuteMsg as WithdrawalVoucherExecuteMsg, Extension as WithdrawalVoucherExtension,
    InstantiateMsg as WithdrawalVoucherInstantiateMsg,
};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies_with_api(&[]);
    let msg = InstantiateMsg {
        withdrawal_voucher_code_id: 0,
        unbonding_period: 123,
        token_metadata: DenomMetadata {
            description: "description".to_string(),
            denom_units: vec![],
            base: "base".to_string(),
            display: "display".to_string(),
            name: "name".to_string(),
            symbol: "symbol".to_string(),
            uri: "uri".to_string(),
            uri_hash: "uri_hash".to_string(),
        },
        subdenom: "subdenom".to_string(),
        exponent: 6u32,
    };
    deps.querier.add_stargate_query_response(
        "/cosmos.wasm.v1.Query/QueryCodeRequest",
        |data| -> cosmwasm_std::Binary {
            let mut y = vec![0; 32];
            y[..data.len()].copy_from_slice(data);
            to_json_binary(&cosmwasm_std::CodeInfoResponse::new(
                from_json(data).unwrap(),
                "creator".to_string(),
                cosmwasm_std::HexBinary::from(y.as_slice()),
            ))
            .unwrap()
        },
    );
    let res = instantiate(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        msg,
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_submessages(vec![
                SubMsg {
                    id: 0,
                    gas_limit: None,
                    reply_on: ReplyOn::Never,
                    msg: CosmosMsg::Wasm(WasmMsg::Instantiate2 {
                        admin: Some("owner".to_string()),
                        code_id: 0,
                        label: "drop-staking-withdrawal-manager".to_string(),
                        msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
                            name: "Drop NTRN Voucher".to_string(),
                            symbol: "DROPV".to_string(),
                            minter: "cosmos2contract".to_string(),
                        })
                        .unwrap(),
                        funds: vec![],
                        salt: Binary::from(SALT.as_bytes())
                    },)
                },
                SubMsg {
                    id: 1,
                    gas_limit: None,
                    reply_on: ReplyOn::Success,
                    msg: CosmosMsg::Custom(NeutronMsg::CreateDenom {
                        subdenom: "subdenom".to_string()
                    }),
                }
            ])
            .add_event(
                Event::new("crates.io:drop-staking__drop-ntrn-derivative-core-instantiate")
                    .add_attributes(vec![
                        attr("owner", "owner"),
                        attr("denom", "subdenom"),
                        attr("withdrawal_voucher_contract", "some_humanized_address")
                    ])
            )
    )
}
