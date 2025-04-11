use crate::contract::{CLAIM_REWARDS_REPLY_ID, CONTRACT_NAME};

use cosmwasm_std::{
    coin, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal256, DelegationResponse, DepsMut, Event,
    FullDelegation, Response, StakingMsg, StdError, SubMsg, Timestamp, Uint128, Uint64, WasmMsg,
};
use cw_utils::PaymentError;
use drop_helpers::testing::mock_dependencies;

use drop_puppeteer_base::peripheral_hook::{
    ReceiverExecuteMsg, ResponseHookMsg, ResponseHookSuccessMsg, Transaction,
};
use drop_staking_base::state::{
    puppeteer::{Delegations, DropDelegation},
    puppeteer_native::{
        unbonding_delegations::{
            QueryDelegatorUnbondingDelegationsResponse, UnbondingDelegationEntry,
            UnbondingDelegationNative,
        },
        Config, ConfigOptional, Delegation, DelegationResponseNative, PageResponse, CONFIG,
        REWARDS_WITHDRAW_ADDR,
    },
};
use drop_staking_base::{
    msg::puppeteer_native::InstantiateMsg,
    state::puppeteer_native::QueryDelegatorDelegationsResponse,
};
use neutron_sdk::{
    bindings::{msg::IbcFee, query::NeutronQuery},
    interchain_queries::v045::types::Balances,
    query::min_ibc_fee::MinIbcFeeResponse,
};

use std::vec;

use crate::contract::DEFAULT_DENOM;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        owner: Some("owner".to_string()),
        allowed_senders: vec!["allowed_sender1".to_string(), "allowed_sender2".to_string()],
        distribution_module_contract: "distribution_module".to_string(),
    };
    let env = mock_env();
    let res =
        crate::contract::instantiate(deps.as_mut(), env, mock_info("sender", &[]), msg).unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-puppeteer-native-instantiate").add_attributes(
                vec![
                    ("owner", "owner"),
                    ("distribution_module_contract", "distribution_module"),
                    ("allowed_senders", "allowed_sender1,allowed_sender2"),
                ]
            )
        )
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, get_base_config());
    assert_eq!(
        cosmwasm_std::Addr::unchecked("owner"),
        cw_ownable::get_ownership(deps.as_mut().storage)
            .unwrap()
            .owner
            .unwrap()
    );
}

#[test]
fn test_execute_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_base_config())
        .unwrap();
    let msg = drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateConfig {
        new_config: ConfigOptional {
            allowed_senders: Some(vec!["new_allowed_sender".to_string()]),
            distribution_module_contract: Some("distribution_module".to_string()),
        },
    };
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    )
}

#[test]
fn test_execute_update_config() {
    let mut deps = mock_dependencies(&[]);
    CONFIG
        .save(deps.as_mut().storage, &get_base_config())
        .unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                allowed_senders: Some(vec!["new_allowed_sender".to_string()]),
                distribution_module_contract: Some("distribution_module".to_string()),
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-puppeteer-native-config_update")
                .add_attributes(vec![
                    ("allowed_senders", "1"),
                    ("distribution_module_contract", "distribution_module"),
                ])
        )
    );

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            allowed_senders: vec![Addr::unchecked("new_allowed_sender")],
            distribution_module_contract: Addr::unchecked("distribution_module"),
        }
    );
}

#[test]
fn test_execute_setup_protocol_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::SetupProtocol {
            rewards_withdraw_address: "rewards_withdraw_address".to_string(),
        },
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_setup_protocol() {
    let mut deps = mock_dependencies(&[]);

    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender1", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::SetupProtocol {
            rewards_withdraw_address: "rewards_withdraw_address".to_string(),
        },
    )
    .unwrap();

    let withdraw_address = REWARDS_WITHDRAW_ADDR.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        withdraw_address,
        Addr::unchecked("rewards_withdraw_address")
    );

    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-puppeteer-native-execute_setup_protocol")
                .add_attributes(vec![(
                    "rewards_withdraw_address",
                    "rewards_withdraw_address"
                ),])
        )
    );
}

#[test]
fn test_execute_undelegate_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Undelegate {
            batch_id: 0u128,
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_undelegate() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("allowed_sender1", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Undelegate {
            batch_id: 0u128,
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_messages(vec![
            CosmosMsg::Staking(StakingMsg::Undelegate {
                validator: "valoper1".to_string(),
                amount: cosmwasm_std::Coin::new(1000u128, DEFAULT_DENOM.to_string()),
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "some_reply_to".to_string(),
                msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
                    ResponseHookMsg::Success(ResponseHookSuccessMsg {
                        transaction: Transaction::Undelegate {
                            interchain_account_id: env.contract.address.to_string(),
                            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
                            denom: DEFAULT_DENOM.to_string(),
                            batch_id: 0u128,
                        },
                        local_height: env.block.height,
                        remote_height: env.block.height,
                    }),
                ))
                .unwrap(),
                funds: vec![],
            })
        ])
    );
}

#[test]
fn test_execute_delegate_no_funds() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("allowed_sender1", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Delegate {
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::PaymentError(PaymentError::NoFunds {})
    );
}

#[test]
fn test_execute_delegate_diff_funds() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(
            "allowed_sender1",
            &[Coin::new(1001u128, DEFAULT_DENOM.to_string())],
        ),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Delegate {
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::InvalidFunds {
            reason: "funds to stake and the attached funds must equal".to_string()
        }
    );
}

#[test]
fn test_execute_delegate() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(
            "allowed_sender1",
            &[Coin::new(1000u128, DEFAULT_DENOM.to_string())],
        ),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Delegate {
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_messages(vec![
                CosmosMsg::Staking(StakingMsg::Delegate {
                    validator: "valoper1".to_string(),
                    amount: cosmwasm_std::Coin::new(1000u128, "untrn".to_string()),
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "some_reply_to".to_string(),
                    msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
                        ResponseHookMsg::Success(ResponseHookSuccessMsg {
                            transaction: Transaction::Stake {
                                amount: Uint128::from(1000u128),
                            },
                            local_height: env.block.height,
                            remote_height: env.block.height,
                        }),
                    ))
                    .unwrap(),
                    funds: vec![],
                })
            ])
            .add_event(
                Event::new("crates.io:drop-staking__drop-puppeteer-native-stake")
                    .add_attributes(vec![("action", "stake"), ("amount_to_stake", "1000")])
            )
    );
}

#[test]
fn test_execute_redelegate_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("not_an_owner", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Redelegate {
            amount: Some(Uint128::from(1000u128)),
            src_validator: "src_validator".to_string(),
            dst_validator: "dst_validator".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    )
}

#[test]
fn test_execute_redelegate() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("owner", &[Coin::new(1000u128, DEFAULT_DENOM.to_string())]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Redelegate {
            amount: Some(Uint128::from(1000u128)),
            src_validator: "src_validator".to_string(),
            dst_validator: "dst_validator".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_messages(vec![CosmosMsg::Staking(StakingMsg::Redelegate {
                src_validator: "src_validator".to_string(),
                dst_validator: "dst_validator".to_string(),
                amount: cosmwasm_std::Coin::new(1000u128, DEFAULT_DENOM.to_string()),
            }),])
            .add_event(
                Event::new("crates.io:drop-staking__drop-puppeteer-native-redelegate")
                    .add_attributes(vec![
                        ("action", "redelegate"),
                        ("amount", "1000"),
                        ("src_validator", "src_validator"),
                        ("dst_validator", "dst_validator")
                    ])
            )
    );
}

#[test]
fn test_execute_redelegate_no_amount() {
    let mut deps = mock_dependencies(&[]);

    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    deps.querier.add_staking_query_response(
        "src_validator".to_string(),
        DelegationResponse {
            delegation: Some(FullDelegation {
                amount: Coin::new(1000u128, "untrn".to_string()),
                delegator: env.contract.clone().address,
                validator: "src_validator".to_string(),
                can_redelegate: Coin::new(1000u128, "base_denom".to_string()),
                accumulated_rewards: vec![],
            }),
        },
    );

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("owner", &[Coin::new(1000u128, DEFAULT_DENOM.to_string())]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Redelegate {
            amount: None,
            src_validator: "src_validator".to_string(),
            dst_validator: "dst_validator".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_messages(vec![CosmosMsg::Staking(StakingMsg::Redelegate {
                src_validator: "src_validator".to_string(),
                dst_validator: "dst_validator".to_string(),
                amount: cosmwasm_std::Coin::new(1000u128, DEFAULT_DENOM.to_string()),
            }),])
            .add_event(
                Event::new("crates.io:drop-staking__drop-puppeteer-native-redelegate")
                    .add_attributes(vec![
                        ("action", "redelegate"),
                        ("amount", "1000"),
                        ("src_validator", "src_validator"),
                        ("dst_validator", "dst_validator")
                    ])
            )
    );
}

#[test]
fn test_execute_redelegate_no_amount_zero_amount() {
    let mut deps = mock_dependencies(&[]);

    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    deps.querier.add_staking_query_response(
        "src_validator".to_string(),
        DelegationResponse {
            delegation: Some(FullDelegation {
                amount: Coin::new(0u128, "untrn".to_string()),
                delegator: env.contract.clone().address,
                validator: "src_validator".to_string(),
                can_redelegate: Coin::new(0u128, "base_denom".to_string()),
                accumulated_rewards: vec![],
            }),
        },
    );

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("owner", &[Coin::new(1000u128, DEFAULT_DENOM.to_string())]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Redelegate {
            amount: None,
            src_validator: "src_validator".to_string(),
            dst_validator: "dst_validator".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::InvalidFunds {
            reason: "amount must be greater than 0".to_string()
        }
    );
}

#[test]
fn test_execute_redelegate_zero_amount() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    let env: cosmwasm_std::Env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("owner", &[Coin::new(1000u128, DEFAULT_DENOM.to_string())]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::Redelegate {
            amount: Some(Uint128::zero()),
            src_validator: "src_validator".to_string(),
            dst_validator: "dst_validator".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::InvalidFunds {
            reason: "amount must be greater than 0".to_string()
        }
    )
}

#[test]
fn test_execute_claim_rewards_and_optionaly_transfer_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators: vec!["validator1".to_string(), "validator2".to_string()],
            transfer: Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg {
                batch_ids: vec![0u128, 1u128, 2u128],
                emergency: true,
                amount: Uint128::from(123u64),
                recipient: "some_recipient".to_string(),
            }),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_claim_rewards_and_optionaly_transfer() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    let env = mock_env();

    let transfer = Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg {
        batch_ids: vec![0u128, 1u128, 2u128],
        emergency: true,
        amount: Uint128::from(123u64),
        recipient: "some_recipient".to_string(),
    });

    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info("allowed_sender1", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators: vec!["validator1".to_string(), "validator2".to_string()],
            transfer: transfer.clone(),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res,
        cosmwasm_std::Response::new()
            .add_messages(vec![
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: "some_recipient".to_string(),
                    amount: vec![cosmwasm_std::Coin::new(123u128, DEFAULT_DENOM.to_string())],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: "some_reply_to".to_string(),
                    msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
                        ResponseHookMsg::Success(ResponseHookSuccessMsg {
                            transaction: Transaction::ClaimRewardsAndOptionalyTransfer {
                                interchain_account_id: env.contract.address.to_string(),
                                validators: vec![
                                    "validator1".to_string(),
                                    "validator2".to_string()
                                ],
                                denom: DEFAULT_DENOM.to_string(),
                                transfer,
                            },
                            local_height: env.block.height,
                            remote_height: env.block.height,
                        }),
                    ))
                    .unwrap(),
                    funds: vec![],
                })
            ])
            .add_submessage(SubMsg::reply_on_error(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "distribution_module".to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::neutron_distribution_mock::ExecuteMsg::ClaimRewards {
                        to_address: Some("rewards_withdraw_address".to_string())
                    }
                )
                .unwrap(),
                funds: vec![],
            }), CLAIM_REWARDS_REPLY_ID)),
    );
}

fn get_base_config() -> Config {
    Config {
        allowed_senders: vec![
            Addr::unchecked("allowed_sender1"),
            Addr::unchecked("allowed_sender2"),
        ],
        distribution_module_contract: Addr::unchecked("distribution_module"),
    }
}

fn base_init(deps_mut: &mut DepsMut<NeutronQuery>) {
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    CONFIG.save(deps_mut.storage, &get_base_config()).unwrap();
    REWARDS_WITHDRAW_ADDR
        .save(
            deps_mut.storage,
            &Addr::unchecked("rewards_withdraw_address"),
        )
        .unwrap();
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: Some(cw_ownable::Expiration::Never {}),
            },
        ),
    )
    .unwrap();
    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("new_owner", &[]),
        drop_staking_base::msg::puppeteer_native::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::AcceptOwnership {},
        ),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Ownership {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(cosmwasm_std::Addr::unchecked("new_owner".to_string())),
            pending_expiry: None,
            pending_owner: None
        }
    );
}

#[test]
fn test_query_extension_delegations_none() {
    let mut deps = mock_dependencies(&[]);

    deps.querier
        .add_stargate_query_response("/cosmos.staking.v1beta1.Query/DelegatorDelegations", |_| {
            cosmwasm_std::ContractResult::Err("No data".to_string())
        });

    let env = mock_env();

    let query_res_err = crate::contract::query(
        deps.as_ref(),
        env.clone(),
        drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
            msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Delegations {},
        },
    )
    .unwrap_err();

    assert_eq!(
        query_res_err,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Querier contract error: No data"
        ))
    )
}

#[test]
fn test_query_extension_delegations_some_one_page() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    deps.querier.add_stargate_query_response(
        "/cosmos.staking.v1beta1.Query/DelegatorDelegations",
        |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&QueryDelegatorDelegationsResponse {
                    delegation_responses: vec![
                        DelegationResponseNative {
                            delegation: Delegation {
                                delegator_address: Addr::unchecked("delegator1"),
                                validator_address: "validator1".to_string(),
                                shares: Decimal256::from_ratio(
                                    cosmwasm_std::Uint256::from(0u64),
                                    cosmwasm_std::Uint256::from(1u64),
                                ),
                            },
                            balance: cosmwasm_std::Coin::new(100, "denom1"),
                        },
                        DelegationResponseNative {
                            delegation: Delegation {
                                delegator_address: Addr::unchecked("delegator2"),
                                validator_address: "validator2".to_string(),
                                shares: Decimal256::from_ratio(
                                    cosmwasm_std::Uint256::from(0u64),
                                    cosmwasm_std::Uint256::from(1u64),
                                ),
                            },
                            balance: cosmwasm_std::Coin::new(100, "denom2"),
                        },
                    ],
                    pagination: PageResponse {
                        next_key: None,
                        total: Uint128::from(2u64),
                    },
                })
                .unwrap(),
            )
        },
    );

    let env = mock_env();

    let query_res: drop_staking_base::msg::puppeteer::DelegationsResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            env.clone(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Delegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::DelegationsResponse {
            delegations: Delegations {
                delegations: vec![
                    DropDelegation {
                        delegator: Addr::unchecked("delegator1"),
                        validator: "validator1".to_string(),
                        amount: cosmwasm_std::Coin::new(100, "denom1"),
                        share_ratio: Decimal256::from_ratio(
                            cosmwasm_std::Uint256::from(0u64),
                            cosmwasm_std::Uint256::from(1u64),
                        ),
                    },
                    DropDelegation {
                        delegator: Addr::unchecked("delegator2"),
                        validator: "validator2".to_string(),
                        amount: cosmwasm_std::Coin::new(100, "denom2"),
                        share_ratio: Decimal256::from_ratio(
                            cosmwasm_std::Uint256::from(0u64),
                            cosmwasm_std::Uint256::from(1u64),
                        ),
                    },
                ]
            },
            remote_height: env.block.height,
            local_height: env.block.height,
            timestamp: env.block.time,
        }
    );
}

#[test]
fn test_query_extension_delegations_some_two_pages() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    deps.querier.add_stargate_query_response(
        "/cosmos.staking.v1beta1.Query/DelegatorDelegations",
        |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&QueryDelegatorDelegationsResponse {
                    delegation_responses: vec![
                        DelegationResponseNative {
                            delegation: Delegation {
                                delegator_address: Addr::unchecked("delegator1"),
                                validator_address: "validator1".to_string(),
                                shares: Decimal256::from_ratio(
                                    cosmwasm_std::Uint256::from(0u64),
                                    cosmwasm_std::Uint256::from(1u64),
                                ),
                            },
                            balance: cosmwasm_std::Coin::new(100, "denom1"),
                        },
                        DelegationResponseNative {
                            delegation: Delegation {
                                delegator_address: Addr::unchecked("delegator2"),
                                validator_address: "validator2".to_string(),
                                shares: Decimal256::from_ratio(
                                    cosmwasm_std::Uint256::from(0u64),
                                    cosmwasm_std::Uint256::from(1u64),
                                ),
                            },
                            balance: cosmwasm_std::Coin::new(100, "denom2"),
                        },
                    ],
                    pagination: PageResponse {
                        next_key: Some(vec![0u8]),
                        total: Uint128::from(2u64),
                    },
                })
                .unwrap(),
            )
        },
    );

    deps.querier.add_stargate_query_response(
        "/cosmos.staking.v1beta1.Query/DelegatorDelegations",
        |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&QueryDelegatorDelegationsResponse {
                    delegation_responses: vec![DelegationResponseNative {
                        delegation: Delegation {
                            delegator_address: Addr::unchecked("delegator3"),
                            validator_address: "validator3".to_string(),
                            shares: Decimal256::from_ratio(
                                cosmwasm_std::Uint256::from(0u64),
                                cosmwasm_std::Uint256::from(1u64),
                            ),
                        },
                        balance: cosmwasm_std::Coin::new(100, "denom3"),
                    }],
                    pagination: PageResponse {
                        next_key: None,
                        total: Uint128::from(2u64),
                    },
                })
                .unwrap(),
            )
        },
    );

    let env = mock_env();

    let query_res: drop_staking_base::msg::puppeteer::DelegationsResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            env.clone(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Delegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::DelegationsResponse {
            delegations: Delegations {
                delegations: vec![
                    DropDelegation {
                        delegator: Addr::unchecked("delegator1"),
                        validator: "validator1".to_string(),
                        amount: cosmwasm_std::Coin::new(100, "denom1"),
                        share_ratio: Decimal256::from_ratio(
                            cosmwasm_std::Uint256::from(0u64),
                            cosmwasm_std::Uint256::from(1u64),
                        ),
                    },
                    DropDelegation {
                        delegator: Addr::unchecked("delegator2"),
                        validator: "validator2".to_string(),
                        amount: cosmwasm_std::Coin::new(100, "denom2"),
                        share_ratio: Decimal256::from_ratio(
                            cosmwasm_std::Uint256::from(0u64),
                            cosmwasm_std::Uint256::from(1u64),
                        ),
                    },
                    DropDelegation {
                        delegator: Addr::unchecked("delegator3"),
                        validator: "validator3".to_string(),
                        amount: cosmwasm_std::Coin::new(100, "denom3"),
                        share_ratio: Decimal256::from_ratio(
                            cosmwasm_std::Uint256::from(0u64),
                            cosmwasm_std::Uint256::from(1u64),
                        ),
                    },
                ]
            },
            remote_height: env.block.height,
            local_height: env.block.height,
            timestamp: env.block.time,
        }
    );
}

#[test]
fn test_query_extension_balances_none() {
    let mut deps = mock_dependencies(&[]);

    base_init(&mut deps.as_mut());

    let env = mock_env();

    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            env.clone(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Balances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances {
                coins: vec![coin(0, DEFAULT_DENOM)]
            },
            remote_height: env.block.height,
            local_height: env.block.height,
            timestamp: env.block.time,
        }
    );
}

#[test]
fn test_query_extension_balances_some() {
    let coins = vec![cosmwasm_std::Coin::new(123u128, DEFAULT_DENOM.to_string())];

    let mut deps = mock_dependencies(&coins);
    base_init(&mut deps.as_mut());

    let env = mock_env();

    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        crate::contract::query(
            deps.as_ref(),
            env.clone(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::Balances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances { coins },
            remote_height: env.block.height,
            local_height: env.block.height,
            timestamp: env.block.time,
        }
    );
}

// #[test]
// fn test_query_non_native_rewards_balances() {
//     let coins = vec![
//         cosmwasm_std::Coin::new(123u128, "denom1".to_string()),
//         cosmwasm_std::Coin::new(123u128, "denom2".to_string()),
//     ];

//     let mut deps = mock_dependencies(&coins);
//     base_init(&mut deps.as_mut());

//     NON_NATIVE_REWARD_BALANCES
//         .save(
//             deps.as_mut().storage,
//             &BalancesAndDelegationsState {
//                 data: drop_staking_base::msg::puppeteer::MultiBalances {
//                     coins: coins.clone(),
//                 },
//                 remote_height: 1u64,
//                 local_height: 2u64,
//                 timestamp: Timestamp::default(),
//                 collected_chunks: vec![],
//             },
//         )
//         .unwrap();
//     let env = mock_env();

//     let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
//         crate::contract::query(
//             deps.as_ref(),
//             env.clone(),
//             drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
//                 msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::NonNativeRewardsBalances {},
//             },
//         )
//         .unwrap(),
//     )
//     .unwrap();
//     assert_eq!(
//         query_res,
//         drop_staking_base::msg::puppeteer::BalancesResponse {
//             balances: Balances { coins },
//             remote_height: env.block.height,
//             local_height: env.block.height,
//             timestamp: env.block.time,
//         }
//     );
// }

#[test]
fn test_unbonding_delegations_one_page() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    deps.querier.add_stargate_query_response(
        "/cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations",
        |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&QueryDelegatorUnbondingDelegationsResponse {
                    unbonding_responses: vec![
                        UnbondingDelegationNative {
                            delegator_address: "delegator_address".to_string(),
                            validator_address: "validator_address1".to_string(),
                            entries: vec![UnbondingDelegationEntry {
                                balance: Uint128::zero(),
                                completion_time: Some("2024-12-12T13:00:42Z".to_string()),
                                creation_height: Uint64::zero(),
                                initial_balance: Uint128::zero(),
                                unbonding_id: Uint128::zero(),
                                unbonding_on_hold_ref_count: Uint128::zero(),
                            }],
                        },
                        UnbondingDelegationNative {
                            delegator_address: "delegator_address".to_string(),
                            validator_address: "validator_address2".to_string(),
                            entries: vec![],
                        },
                    ],
                    pagination: PageResponse {
                        next_key: None,
                        total: Uint128::from(2u64),
                    },
                })
                .unwrap(),
            )
        },
    );

    let query_res: Vec<drop_puppeteer_base::state::UnbondingDelegation> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::UnbondingDelegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        vec![
            drop_puppeteer_base::state::UnbondingDelegation {
                validator_address: "validator_address1".to_string(),
                query_id: 0u64,
                unbonding_delegations: vec![
                    neutron_sdk::interchain_queries::v047::types::UnbondingEntry {
                        balance: Uint128::from(0u64),
                        completion_time: Some(Timestamp::from_seconds(1734008442)),
                        creation_height: 0u64,
                        initial_balance: Uint128::from(0u64),
                    },
                ],
                last_updated_height: 0u64,
            },
            drop_puppeteer_base::state::UnbondingDelegation {
                validator_address: "validator_address2".to_string(),
                query_id: 0u64,
                unbonding_delegations: vec![],
                last_updated_height: 0u64,
            }
        ]
    );
}

#[test]
fn test_unbonding_delegations_two_pages() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());

    deps.querier.add_stargate_query_response(
        "/cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations",
        |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&QueryDelegatorUnbondingDelegationsResponse {
                    unbonding_responses: vec![
                        UnbondingDelegationNative {
                            delegator_address: "delegator_address".to_string(),
                            validator_address: "validator_address1".to_string(),
                            entries: vec![UnbondingDelegationEntry {
                                balance: Uint128::zero(),
                                completion_time: Some("2024-12-12T13:00:42Z".to_string()),
                                creation_height: Uint64::zero(),
                                initial_balance: Uint128::zero(),
                                unbonding_id: Uint128::zero(),
                                unbonding_on_hold_ref_count: Uint128::zero(),
                            }],
                        },
                        UnbondingDelegationNative {
                            delegator_address: "delegator_address".to_string(),
                            validator_address: "validator_address2".to_string(),
                            entries: vec![],
                        },
                    ],
                    pagination: PageResponse {
                        next_key: Some(vec![0u8]),
                        total: Uint128::from(2u64),
                    },
                })
                .unwrap(),
            )
        },
    );

    deps.querier.add_stargate_query_response(
        "/cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations",
        |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&QueryDelegatorUnbondingDelegationsResponse {
                    unbonding_responses: vec![UnbondingDelegationNative {
                        delegator_address: "delegator_address".to_string(),
                        validator_address: "validator_address3".to_string(),
                        entries: vec![UnbondingDelegationEntry {
                            balance: Uint128::zero(),
                            completion_time: Some("2024-12-12T13:00:42Z".to_string()),
                            creation_height: Uint64::zero(),
                            initial_balance: Uint128::zero(),
                            unbonding_id: Uint128::zero(),
                            unbonding_on_hold_ref_count: Uint128::zero(),
                        }],
                    }],
                    pagination: PageResponse {
                        next_key: None,
                        total: Uint128::from(2u64),
                    },
                })
                .unwrap(),
            )
        },
    );

    let query_res: Vec<drop_puppeteer_base::state::UnbondingDelegation> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_staking_base::msg::puppeteer_native::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer_native::QueryExtMsg::UnbondingDelegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        vec![
            drop_puppeteer_base::state::UnbondingDelegation {
                validator_address: "validator_address1".to_string(),
                query_id: 0u64,
                unbonding_delegations: vec![
                    neutron_sdk::interchain_queries::v047::types::UnbondingEntry {
                        balance: Uint128::from(0u64),
                        completion_time: Some(Timestamp::from_seconds(1734008442)),
                        creation_height: 0u64,
                        initial_balance: Uint128::from(0u64),
                    },
                ],
                last_updated_height: 0u64,
            },
            drop_puppeteer_base::state::UnbondingDelegation {
                validator_address: "validator_address2".to_string(),
                query_id: 0u64,
                unbonding_delegations: vec![],
                last_updated_height: 0u64,
            },
            drop_puppeteer_base::state::UnbondingDelegation {
                validator_address: "validator_address3".to_string(),
                query_id: 0u64,
                unbonding_delegations: vec![
                    neutron_sdk::interchain_queries::v047::types::UnbondingEntry {
                        balance: Uint128::from(0u64),
                        completion_time: Some(Timestamp::from_seconds(1734008442)),
                        creation_height: 0u64,
                        initial_balance: Uint128::from(0u64),
                    },
                ],
                last_updated_height: 0u64,
            },
        ]
    );
}

#[test]
fn test_migrate_wrong_contract() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();

    cw2::set_contract_version(deps_mut.storage, "wrong_contract_name", "0.0.1").unwrap();

    let res = crate::contract::migrate(
        deps.as_mut(),
        mock_env(),
        drop_staking_base::msg::puppeteer_native::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: CONTRACT_NAME.to_string()
        }
    )
}

fn get_standard_fees() -> IbcFee {
    IbcFee {
        recv_fee: vec![],
        ack_fee: cosmwasm_std::coins(100, "untrn"),
        timeout_fee: cosmwasm_std::coins(200, "untrn"),
    }
}
