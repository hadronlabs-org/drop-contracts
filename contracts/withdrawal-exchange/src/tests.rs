use crate::contract::{execute, query, UNBOND_MARK};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Event, QueryRequest, Response, Uint128,
};
use cw721::{Cw721ReceiveMsg, NftInfoResponse};
use drop_helpers::testing::mock_dependencies;
use drop_staking_base::msg::withdrawal_exchange::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use drop_staking_base::state::withdrawal_exchange::{Config, CONFIG};
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::query::token_factory::FullDenomResponse;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        withdrawal_token_address: "withdrawal_token_contract".to_string(),
        withdrawal_voucher_address: "withdrawal_voucher_contract".to_string(),
        denom_prefix: "denom".to_string(),
        owner: "owner".to_string(),
    };
    let env = mock_env();
    let res =
        crate::contract::instantiate(deps.as_mut(), env, mock_info("sender", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("drop-withdrawal-exchange-instantiate").add_attributes(vec![
                ("action", "instantiate"),
                ("withdrawal_token", "withdrawal_token_contract"),
                ("withdrawal_voucher", "withdrawal_voucher_contract"),
                ("denom_prefix", "denom")
            ])
        )
    );
    assert_eq!(
        Addr::unchecked("owner"),
        cw_ownable::get_ownership(deps.as_mut().storage)
            .unwrap()
            .owner
            .unwrap()
    );
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                withdrawal_token_contract: Addr::unchecked("withdrawal_token"),
                withdrawal_voucher_contract: Addr::unchecked("withdrawal_voucher"),
                denom_prefix: "denom".to_string(),
            },
        )
        .unwrap();

    let response = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    assert_eq!(
        response,
        to_json_binary(&ConfigResponse {
            withdrawal_token_address: "withdrawal_token".to_string(),
            withdrawal_voucher_address: "withdrawal_voucher".to_string(),
            denom_prefix: "denom".to_string(),
        })
        .unwrap()
    );
}

#[test]
fn test_exchange() {
    let mut deps = mock_dependencies(&[]);

    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                withdrawal_token_contract: Addr::unchecked("withdrawal_token_contract"),
                withdrawal_voucher_contract: Addr::unchecked("withdrawal_voucher_contract"),
                denom_prefix: "denom".to_string(),
            },
        )
        .unwrap();

    deps.querier
        .add_wasm_query_response("withdrawal_voucher_contract", |_| {
            to_json_binary(&NftInfoResponse {
                token_uri: None,
                extension: drop_staking_base::state::withdrawal_voucher::Metadata {
                    name: "nft_name".to_string(),
                    description: None,
                    attributes: None,
                    batch_id: "0".to_string(),
                    amount: Uint128::from(1000u128),
                },
            })
            .unwrap()
        });

    deps.querier
        .add_custom_query_response(|request| match request {
            QueryRequest::Custom(NeutronQuery::FullDenom {
                creator_addr,
                subdenom,
            }) => {
                assert_eq!(creator_addr, MOCK_CONTRACT_ADDR);
                assert_eq!(subdenom, &format!("denom:{}:0", UNBOND_MARK));
                to_json_binary(&FullDenomResponse {
                    denom: format!(
                        "factory/{}/denom_prefix:{}:0",
                        MOCK_CONTRACT_ADDR, UNBOND_MARK
                    )
                    .to_string(),
                })
                .unwrap()
            }
            _ => unimplemented!(),
        });

    deps.querier
        .add_wasm_query_response("withdrawal_voucher_contract", |_| {
            to_json_binary(&vec![NftInfoResponse {
                token_uri: None,
                extension: &drop_staking_base::state::withdrawal_voucher::Metadata {
                    name: "nft_name".to_string(),
                    description: None,
                    attributes: None,
                    batch_id: "0".to_string(),
                    amount: Uint128::from(1000u128),
                },
            }])
            .unwrap()
        });

    let response = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("withdrawal_voucher_contract", &[]),
        ExecuteMsg::Exchange(Cw721ReceiveMsg {
            sender: "sender".to_string(),
            token_id: "0_sender_0".to_string(),
            msg: to_json_binary(
                &drop_staking_base::msg::withdrawal_exchange::ReceiveNftMsg::Withdraw {
                    receiver: None,
                },
            )
            .unwrap(),
        }),
    )
    .unwrap();

    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "sender".to_string(),
            amount: vec![Coin {
                denom: format!(
                    "factory/{}/denom_prefix:{}:0",
                    MOCK_CONTRACT_ADDR, UNBOND_MARK
                )
                .to_string(),
                amount: Uint128::from(1000u128),
            }],
        })
    );
}
