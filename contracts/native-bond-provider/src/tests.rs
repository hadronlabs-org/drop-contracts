use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{
    attr, coins, from_json,
    testing::{message_info, mock_env},
    to_json_binary, Addr, BalanceResponse, Coin, CosmosMsg, Decimal, Event, Response, SubMsg,
    Uint128, WasmMsg,
};
use cw_ownable::{Action, Ownership};
use cw_utils::PaymentError;
use drop_helpers::{
    ica::IcaState,
    testing::{mock_dependencies, mock_state_query},
};
use drop_staking_base::state::native_bond_provider::{
    Config, ConfigOptional, ReplyMsg, TxState, CONFIG, NON_STAKED_BALANCE, TX_STATE,
};
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::RequestPacketTimeoutHeight,
};

fn get_default_config(api: MockApi) -> Config {
    Config {
        factory_contract: api.addr_make("factory_contract"),
        base_denom: "base_denom".to_string(),
        min_ibc_transfer: Uint128::from(100u128),
        min_stake_amount: Uint128::from(100u128),
        port_id: "port_id".to_string(),
        transfer_channel_id: "transfer_channel_id".to_string(),
        timeout: 100u64,
    }
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let response = crate::contract::instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("admin"), &[]),
        drop_staking_base::msg::native_bond_provider::InstantiateMsg {
            owner: api.addr_make("owner").to_string(),
            base_denom: "base_denom".to_string(),
            factory_contract: api.addr_make("factory_contract").to_string(),
            min_ibc_transfer: Uint128::from(100u128),
            min_stake_amount: Uint128::from(100u128),
            port_id: "port_id".to_string(),
            transfer_channel_id: "transfer_channel_id".to_string(),
            timeout: 100u64,
        },
    )
    .unwrap();

    let config = CONFIG.load(deps.as_ref().storage).unwrap();

    assert_eq!(config, get_default_config(api));

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-native-bond-provider-instantiate")
                .add_attributes([
                    attr("factory_contract", api.addr_make("factory_contract")),
                    attr("min_ibc_transfer", Uint128::from(100u128)),
                    attr("min_stake_amount", Uint128::from(100u128)),
                    attr("base_denom", "base_denom"),
                    attr("port_id", "port_id"),
                    attr("transfer_channel_id", "transfer_channel_id"),
                    attr("timeout", "100"),
                ])
        ]
    );
    assert!(response.attributes.is_empty());
}

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(response, to_json_binary(&get_default_config(api)).unwrap());
}

#[test]
fn update_config_wrong_owner() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let deps_mut = deps.as_mut();

    CONFIG
        .save(deps_mut.storage, &get_default_config(api))
        .unwrap();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("core").as_ref()),
    );

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("core1"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                base_denom: Some("base_denom".to_string()),
                factory_contract: Some(api.addr_make("factory_contract")),
                min_ibc_transfer: Some(Uint128::from(100u128)),
                min_stake_amount: Some(Uint128::from(100u128)),
                port_id: Some("port_id".to_string()),
                transfer_channel_id: Some("transfer_channel_id".to_string()),
                timeout: Some(100u64),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn update_config_ok() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("core").as_ref()),
    );

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("core"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                base_denom: Some("base_denom_1".to_string()),
                factory_contract: Some(api.addr_make("factory_contract_1")),
                min_ibc_transfer: Some(Uint128::from(90u128)),
                min_stake_amount: Some(Uint128::from(90u128)),
                port_id: Some("port_id_1".to_string()),
                transfer_channel_id: Some("transfer_channel_id_1".to_string()),
                timeout: Some(90u64),
            },
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.events,
        vec![
            Event::new("crates.io:drop-staking__drop-native-bond-provider-update_config")
                .add_attributes([
                    attr("factory_contract", api.addr_make("factory_contract_1")),
                    attr("base_denom", "base_denom_1"),
                    attr("min_ibc_transfer", Uint128::from(90u128)),
                    attr("min_stake_amount", Uint128::from(90u128)),
                    attr("port_id", "port_id_1"),
                    attr("transfer_channel_id", "transfer_channel_id_1"),
                    attr("timeout", "90"),
                ])
        ]
    );
    assert!(response.attributes.is_empty());

    let config = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::Config {},
    )
    .unwrap();
    assert_eq!(
        config,
        to_json_binary(&Config {
            factory_contract: api.addr_make("factory_contract_1"),
            base_denom: "base_denom_1".to_string(),
            min_ibc_transfer: Uint128::from(90u128),
            min_stake_amount: Uint128::from(90u128),
            port_id: "port_id_1".to_string(),
            transfer_channel_id: "transfer_channel_id_1".to_string(),
            timeout: 90u64,
        })
        .unwrap()
    );
}

#[test]
fn query_ownership() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let deps_mut = deps.as_mut();

    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("core").as_ref()),
    )
    .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::Ownership {},
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&Ownership {
            owner: Some(api.addr_make("core")),
            pending_owner: None,
            pending_expiry: None
        })
        .unwrap()
    );
}

#[test]
fn query_can_bond_ok() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let can_bond = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::CanBond {
            denom: "base_denom".to_string(),
        },
    )
    .unwrap();

    assert_eq!(can_bond, to_json_binary(&true).unwrap());
}

#[test]
fn query_can_bond_false() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let can_bond = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::CanBond {
            denom: "wrong_denom".to_string(),
        },
    )
    .unwrap();

    assert_eq!(can_bond, to_json_binary(&false).unwrap());
}

#[test]
fn query_can_not_process_on_idle_not_in_idle_state() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::native_bond_provider::TxState {
                status: drop_staking_base::state::native_bond_provider::TxStateStatus::InProgress,
                transaction: Some(drop_puppeteer_base::peripheral_hook::Transaction::Stake {
                    amount: Uint128::from(0u64),
                }),
            },
        )
        .unwrap();

    let error = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::InvalidState {
            reason: "tx_state is not idle".to_string()
        }
    );
}

#[test]
fn query_can_process_on_idle_false_if_no_funds_to_process() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();

    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();

    deps.querier.add_bank_query_response(
        "cosmos2contract".to_string(),
        BalanceResponse::new(Coin::new(0u128, "base_denom".to_string())),
    );

    let error = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::NotEnoughToProcessIdle {
            min_stake_amount: Uint128::from(100u128),
            non_staked_balance: Uint128::from(0u128),
            min_ibc_transfer: Uint128::from(100u128),
            pending_coins: Uint128::zero()
        }
    );
}

#[test]
fn query_can_process_on_idle_enough_non_staked_balance() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(100u128))
        .unwrap();

    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();

    let res = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap();

    deps.querier.add_bank_query_response(
        "cosmos2contract".to_string(),
        BalanceResponse::new(Coin::new(0u128, "base_denom".to_string())),
    );

    let res: bool = from_json(res).unwrap();

    assert!(res);
}

#[test]
fn query_can_process_on_idle_enough_contract_balance() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(100u128))
        .unwrap();

    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();

    let res = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::CanProcessOnIdle {},
    )
    .unwrap();

    deps.querier.add_bank_query_response(
        "cosmos2contract".to_string(),
        BalanceResponse::new(Coin::new(100u128, "base_denom".to_string())),
    );

    let res: bool = from_json(res).unwrap();

    assert!(res);
}

#[test]
fn query_token_amount() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: "base_denom".to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::one(),
        },
    )
    .unwrap();

    assert_eq!(
        token_amount,
        to_json_binary(&Uint128::new(100u128)).unwrap()
    );
}

#[test]
fn query_token_amount_half() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: "base_denom".to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::from_atomics(Uint128::from(5u64), 1).unwrap(),
        },
    )
    .unwrap();

    assert_eq!(
        token_amount,
        to_json_binary(&Uint128::new(200u128)).unwrap()
    );
}

#[test]
fn query_token_amount_above_one() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let token_amount = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: "base_denom".to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::from_atomics(Uint128::from(11u64), 1).unwrap(),
        },
    )
    .unwrap();

    assert_eq!(token_amount, to_json_binary(&Uint128::new(90u128)).unwrap());
}

#[test]
fn query_token_amount_wrong_denom() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let error = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::TokensAmount {
            coin: Coin {
                denom: "wrong_denom".to_string(),
                amount: 100u128.into(),
            },
            exchange_rate: Decimal::one(),
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::InvalidDenom {}
    );
}

#[test]
fn update_ownership() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    let deps_mut = deps.as_mut();

    let _result = cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(api.addr_make("core").as_ref()),
    );

    crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("core"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::UpdateOwnership(
            Action::TransferOwnership {
                new_owner: api.addr_make("new_owner").to_string(),
                expiry: None,
            },
        ),
    )
    .unwrap();

    let response = crate::contract::query(
        deps.as_ref(),
        mock_env(),
        drop_staking_base::msg::native_bond_provider::QueryMsg::Ownership {},
    )
    .unwrap();

    assert_eq!(
        response,
        to_json_binary(&Ownership {
            owner: Some(api.addr_make("core")),
            pending_owner: Some(api.addr_make("new_owner")),
            pending_expiry: None
        })
        .unwrap()
    );
}

#[test]
fn process_on_idle_not_in_idle_state() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();

    TX_STATE
        .save(
            deps.as_mut().storage,
            &drop_staking_base::state::native_bond_provider::TxState {
                status: drop_staking_base::state::native_bond_provider::TxStateStatus::InProgress,
                transaction: Some(drop_puppeteer_base::peripheral_hook::Transaction::Stake {
                    amount: Uint128::from(0u64),
                }),
            },
        )
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("core_contract"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::InvalidState {
            reason: "tx_state is not idle".to_string()
        }
    );
}

#[test]
fn process_on_idle_not_core_contract() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("not_core_contract"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::Unauthorized {}
    );
}

#[test]
fn process_on_idle_delegation() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::from(100u128))
        .unwrap();

    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();

    deps.querier
        .add_wasm_query_response("strategy_contract", |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&vec![(
                    api.addr_make("valoper_address").to_string(),
                    Uint128::from(1000u128),
                )])
                .unwrap(),
            )
        });

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("core_contract"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![attr("action", "process_on_idle"),])
            .add_event(Event::new(
                "crates.io:drop-staking__drop-native-bond-provider-process_on_idle"
            ))
            .add_submessage(SubMsg::reply_always(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: api.addr_make("puppeteer_contract").to_string(),
                    msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
                        items: vec![(
                            api.addr_make("valoper_address").to_string(),
                            Uint128::from(1000u128)
                        )],
                        reply_to: api.addr_make("cosmos2contract").to_string()
                    })
                    .unwrap(),
                    funds: vec![],
                }),
                ReplyMsg::Bond.to_reply_id()
            ))
    );
}

#[test]
fn process_on_idle_ibc_transfer() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();

    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();

    deps.querier.add_bank_query_response(
        api.addr_make("cosmos2contract").to_string(),
        BalanceResponse::new(Coin::new(100u128, "base_denom".to_string())),
    );

    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: coins(100, "local_denom"),
                timeout_fee: coins(200, "local_denom"),
            },
        })
        .unwrap()
    });

    deps.querier
        .add_wasm_query_response(api.addr_make("puppeteer_contract").as_str(), |_| {
            cosmwasm_std::ContractResult::Ok(
                to_json_binary(&IcaState::Registered {
                    ica_address: api.addr_make("ica_address").to_string(),
                    port_id: "port_id".to_string(),
                    channel_id: "channel_id".to_string(),
                })
                .unwrap(),
            )
        });

    let mocked_env = mock_env();

    let res = crate::contract::execute(
        deps.as_mut(),
        mocked_env.clone(),
        message_info(&api.addr_make("core_contract"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_attributes(vec![attr("action", "process_on_idle"),])
            .add_event(Event::new(
                "crates.io:drop-staking__drop-native-bond-provider-process_on_idle"
            ))
            .add_submessage(SubMsg::reply_always(
                NeutronMsg::IbcTransfer {
                    source_port: "port_id".to_string(),
                    source_channel: "transfer_channel_id".to_string(),
                    token: Coin::new(100u128, "base_denom"),
                    sender: api.addr_make("cosmos2contract").to_string(),
                    receiver: api.addr_make("ica_address").to_string(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None,
                    },
                    timeout_timestamp: mocked_env.block.time.plus_seconds(100u64).nanos(),
                    memo: "".to_string(),
                    fee: IbcFee {
                        recv_fee: vec![],
                        ack_fee: vec![],
                        timeout_fee: vec![]
                    },
                },
                ReplyMsg::IbcTransfer.to_reply_id()
            ))
    );
}

#[test]
fn process_on_idle_not_allowed_if_no_funds() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    mock_state_query(&mut deps);

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    NON_STAKED_BALANCE
        .save(deps.as_mut().storage, &Uint128::zero())
        .unwrap();

    TX_STATE
        .save(deps.as_mut().storage, &TxState::default())
        .unwrap();

    deps.querier.add_bank_query_response(
        api.addr_make("cosmos2contract").to_string(),
        BalanceResponse::new(Coin::new(0u128, "base_denom".to_string())),
    );

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&api.addr_make("core_contract"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::ProcessOnIdle {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::NotEnoughToProcessIdle {
            min_stake_amount: Uint128::from(100u128),
            non_staked_balance: Uint128::zero(),
            min_ibc_transfer: Uint128::from(100u128),
            pending_coins: Uint128::zero()
        }
    );
}

#[test]
fn execute_bond() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let response = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(
            &Addr::unchecked("core"),
            &[Coin::new(100u128, "base_denom")],
        ),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);

    assert_eq!(
        response,
        Response::new().add_event(
            Event::new("crates.io:drop-staking__drop-native-bond-provider-bond")
                .add_attributes(vec![("received_funds", "100base_denom"),])
        )
    );
}

#[test]
fn execute_bond_wrong_denom() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(
            &Addr::unchecked("core"),
            &[Coin::new(100u128, "wrong_denom")],
        ),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::InvalidDenom {}
    );
}

#[test]
fn execute_bond_no_funds() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked("core"), &[]),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::PaymentError(
            PaymentError::NoFunds {}
        )
    );
}

#[test]
fn execute_bond_multiple_denoms() {
    let mut deps = mock_dependencies(&[]);
    let api = deps.api;

    CONFIG
        .save(deps.as_mut().storage, &get_default_config(api))
        .unwrap();

    let error = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        message_info(
            &Addr::unchecked("core"),
            &[
                Coin::new(100u128, "base_denom"),
                Coin::new(100u128, "second_denom"),
            ],
        ),
        drop_staking_base::msg::native_bond_provider::ExecuteMsg::Bond {},
    )
    .unwrap_err();

    assert_eq!(
        error,
        drop_staking_base::error::native_bond_provider::ContractError::PaymentError(
            PaymentError::MultipleDenoms {}
        )
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
        drop_staking_base::msg::native_bond_provider::MigrateMsg {},
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_staking_base::error::native_bond_provider::ContractError::MigrationError {
            storage_contract_name: "wrong_contract_name".to_string(),
            contract_name: crate::contract::CONTRACT_NAME.to_string()
        }
    )
}
