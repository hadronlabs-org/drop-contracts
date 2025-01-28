use std::collections::HashMap;

use crate::contract::{execute, instantiate, query};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Attribute, Binary, Decimal, Decimal256, Deps, Empty, Env,
    Event, Response, StdResult, Timestamp, Uint128,
};
use cw_multi_test::{custom_app, App, Contract, ContractWrapper, Executor};
use drop_helpers::testing::mock_dependencies;
use drop_puppeteer_base::error::ContractError as PuppeteerContractError;
use drop_puppeteer_base::msg::QueryMsg as PuppeteerQueryMsg;
use drop_staking_base::error::distribution::ContractError as DistributionContractError;
use drop_staking_base::error::factory::ContractError as FactoryContractError;
use drop_staking_base::error::validatorset::ContractError as ValidatorSetContractError;
use drop_staking_base::msg::strategy::QueryMsg;
use drop_staking_base::msg::validatorset::QueryMsg as ValidatorSetQueryMsg;
use drop_staking_base::msg::{
    distribution::QueryMsg as DistributionQueryMsg, factory::QueryMsg as FactoryQueryMsg,
    strategy::InstantiateMsg,
};
use drop_staking_base::state::puppeteer::{Delegations, DropDelegation};
use drop_staking_base::state::strategy::{DENOM, FACTORY_CONTRACT};

const CORE_CONTRACT_ADDR: &str = "core_contract";
const FACTORY_CONTRACT_ADDR: &str = "factory_contract";
const PUPPETEER_CONTRACT_ADDR: &str = "puppeteer_contract";
const VALIDATOR_SET_CONTRACT_ADDR: &str = "validators_set_contract";
const DISTRIBUTION_CONTRACT_ADDR: &str = "distribution_contract";

#[cw_serde]
pub struct EmptyMsg {}

fn instantiate_contract(
    app: &mut App,
    contract: fn() -> Box<dyn Contract<Empty>>,
    label: String,
) -> Addr {
    let contract_id = app.store_code(contract());
    app.instantiate_contract(
        contract_id,
        Addr::unchecked(CORE_CONTRACT_ADDR),
        &EmptyMsg {},
        &[],
        label,
        None,
    )
    .unwrap()
}

fn distribution_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        DistributionQueryMsg,
        DistributionContractError,
        DistributionContractError,
        DistributionContractError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        drop_distribution::contract::query,
    );
    Box::new(contract)
}

fn instantiate_distribution_contract(app: &mut App) -> Addr {
    instantiate_contract(
        app,
        distribution_contract,
        "drop distribution contract".to_string(),
    )
}

fn puppeteer_query(
    _deps: Deps,
    _env: Env,
    msg: PuppeteerQueryMsg<drop_staking_base::msg::puppeteer::QueryExtMsg>,
) -> StdResult<Binary> {
    match msg {
        PuppeteerQueryMsg::Config {} => todo!(),
        PuppeteerQueryMsg::Ica {} => todo!(),
        PuppeteerQueryMsg::TxState {} => todo!(),
        PuppeteerQueryMsg::Transactions {} => todo!(),
        PuppeteerQueryMsg::KVQueryIds {} => todo!(),
        PuppeteerQueryMsg::Ownership {} => todo!(),
        PuppeteerQueryMsg::Extension { msg } => match msg {
            drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {} => {
                let mut delegations_amount: Vec<DropDelegation> = Vec::new();
                for i in 0..3 {
                    let delegation = DropDelegation {
                        validator: format!("valoper{}", i),
                        delegator: Addr::unchecked("delegator".to_owned() + i.to_string().as_str()),
                        amount: cosmwasm_std::Coin {
                            denom: "uatom".to_string(),
                            amount: Uint128::from(100u128),
                        },
                        share_ratio: Decimal256::one(),
                    };
                    delegations_amount.push(delegation);
                }
                let delegations = drop_staking_base::msg::puppeteer::DelegationsResponse {
                    delegations: Delegations {
                        delegations: delegations_amount,
                    },
                    remote_height: 0u64,
                    local_height: 0u64,
                    timestamp: Timestamp::default(),
                };
                Ok(to_json_binary(&delegations)?)
            }
            _ => todo!(),
        },
    }
}

fn puppeteer_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        PuppeteerQueryMsg<drop_staking_base::msg::puppeteer::QueryExtMsg>,
        PuppeteerContractError,
        PuppeteerContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        puppeteer_query,
    );
    Box::new(contract)
}

fn instantiate_puppeteer_contract(app: &mut App) -> Addr {
    instantiate_contract(
        app,
        puppeteer_contract,
        "drop puppeteeer contract".to_string(),
    )
}

fn factory_query(_deps: Deps, _env: Env, msg: FactoryQueryMsg) -> StdResult<Binary> {
    match msg {
        FactoryQueryMsg::State {} => {
            let out = HashMap::from([
                (
                    VALIDATOR_SET_CONTRACT_ADDR.to_string(),
                    "contract1".to_string(),
                ),
                (PUPPETEER_CONTRACT_ADDR.to_string(), "contract2".to_string()),
                (
                    DISTRIBUTION_CONTRACT_ADDR.to_string(),
                    "contract3".to_string(),
                ),
            ]);
            Ok(to_json_binary(&out).unwrap())
        }
        FactoryQueryMsg::PauseInfo {} => todo!(),
        FactoryQueryMsg::Ownership {} => todo!(),
    }
}

fn factory_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        FactoryQueryMsg,
        FactoryContractError,
        FactoryContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        factory_query,
    );
    Box::new(contract)
}

fn instantiate_factory_contract(app: &mut App) -> Addr {
    instantiate_contract(app, factory_contract, "drop factory contract".to_string())
}

fn validator_set_query(_deps: Deps, _env: Env, msg: ValidatorSetQueryMsg) -> StdResult<Binary> {
    match msg {
        ValidatorSetQueryMsg::Ownership {} => todo!(),
        ValidatorSetQueryMsg::Config {} => todo!(),
        ValidatorSetQueryMsg::Validator { valoper: _ } => todo!(),
        ValidatorSetQueryMsg::Validators {} => {
            let mut validators = Vec::new();
            for i in 0..3 {
                let validator = drop_staking_base::state::validatorset::ValidatorInfo {
                    valoper_address: format!("valoper{}", i),
                    weight: 100,
                    on_top: Uint128::zero(),
                    last_processed_remote_height: None,
                    last_processed_local_height: None,
                    last_validated_height: None,
                    last_commission_in_range: None,
                    uptime: Decimal::zero(),
                    tombstone: false,
                    jailed_number: None,
                    init_proposal: None,
                    total_passed_proposals: 0,
                    total_voted_proposals: 0,
                };
                validators.push(validator);
            }
            Ok(to_json_binary(&validators)?)
        }
    }
}

fn validator_set_contract() -> Box<dyn Contract<Empty>> {
    let contract: ContractWrapper<
        EmptyMsg,
        EmptyMsg,
        ValidatorSetQueryMsg,
        ValidatorSetContractError,
        ValidatorSetContractError,
        cosmwasm_std::StdError,
    > = ContractWrapper::new(
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        |_, _, _, _: EmptyMsg| Ok(Response::new()),
        validator_set_query,
    );
    Box::new(contract)
}

fn instantiate_validator_set_contract(app: &mut App) -> Addr {
    instantiate_contract(
        app,
        validator_set_contract,
        "drop validator set contract".to_string(),
    )
}

fn strategy_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn instantiate_strategy_contract(app: &mut App, id: u64, msg: InstantiateMsg) -> Addr {
    app.instantiate_contract(
        id,
        Addr::unchecked(CORE_CONTRACT_ADDR),
        &msg,
        &[],
        "strategy contract",
        None,
    )
    .unwrap()
}

fn mock_app() -> App {
    custom_app(|_r, _a, _s| {})
}

#[test]
fn test_initialization() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        owner: CORE_CONTRACT_ADDR.to_string(),
        factory_contract: "factory_contract".to_string(),
        denom: "uatom".to_string(),
    };

    let info = mock_info(CORE_CONTRACT_ADDR, &[]);
    let res = instantiate(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        msg.clone(),
    )
    .unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("crates.io:drop-staking__drop-strategy-instantiate".to_string())
                .add_attributes(vec![
                    Attribute::new("owner".to_string(), CORE_CONTRACT_ADDR.to_string()),
                    Attribute::new(
                        "factory_contract".to_string(),
                        "factory_contract".to_string(),
                    ),
                    Attribute::new("denom".to_string(), "uatom".to_string()),
                ])
        ]
    );
}

#[test]
fn test_config_query() {
    let mut app = mock_app();
    let strategy_id = app.store_code(strategy_contract());

    let strategy_contract = instantiate_strategy_contract(
        &mut app,
        strategy_id,
        InstantiateMsg {
            owner: CORE_CONTRACT_ADDR.to_string(),
            factory_contract: "factory_contract".to_string(),
            denom: "uatom".to_string(),
        },
    );

    let config: drop_staking_base::msg::strategy::Config = app
        .wrap()
        .query_wasm_smart(strategy_contract.clone(), &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        config,
        drop_staking_base::msg::strategy::Config {
            factory_contract: "factory_contract".to_string(),
            denom: "uatom".to_string(),
        }
    );
}

#[test]
fn test_ideal_deposit_calculation() {
    let mut app = mock_app();
    let factory_contract = instantiate_factory_contract(&mut app);
    let _validator_set_contract = instantiate_validator_set_contract(&mut app);
    let _puppeteer_contract = instantiate_puppeteer_contract(&mut app);
    let _distribution_contract = instantiate_distribution_contract(&mut app);

    let strategy_id = app.store_code(strategy_contract());

    let strategy_contract = instantiate_strategy_contract(
        &mut app,
        strategy_id,
        InstantiateMsg {
            owner: CORE_CONTRACT_ADDR.to_string(),
            factory_contract: factory_contract.to_string(),
            denom: "uatom".to_string(),
        },
    );

    let mut ideal_deposit: Vec<(String, Uint128)> = app
        .wrap()
        .query_wasm_smart(
            strategy_contract,
            &QueryMsg::CalcDeposit {
                deposit: 100u128.into(),
            },
        )
        .unwrap();
    ideal_deposit.sort();

    assert_eq!(
        ideal_deposit,
        vec![
            ("valoper0".to_string(), Uint128::from(34u128)),
            ("valoper1".to_string(), Uint128::from(34u128)),
            ("valoper2".to_string(), Uint128::from(32u128))
        ]
    );
}

#[test]
fn test_ideal_withdraw_calculation() {
    let mut app = mock_app();
    let factory_contract = instantiate_factory_contract(&mut app);
    let _validator_set_contract = instantiate_validator_set_contract(&mut app);
    let _puppeteer_contract = instantiate_puppeteer_contract(&mut app);
    let _distribution_contract = instantiate_distribution_contract(&mut app);

    let strategy_id = app.store_code(strategy_contract());

    let strategy_contract = instantiate_strategy_contract(
        &mut app,
        strategy_id,
        InstantiateMsg {
            owner: CORE_CONTRACT_ADDR.to_string(),
            factory_contract: factory_contract.to_string(),
            denom: "uatom".to_string(),
        },
    );

    let mut ideal_deposit: Vec<(String, Uint128)> = app
        .wrap()
        .query_wasm_smart(
            strategy_contract,
            &QueryMsg::CalcWithdraw {
                withdraw: 100u128.into(),
            },
        )
        .unwrap();
    ideal_deposit.sort();

    assert_eq!(
        ideal_deposit,
        vec![
            ("valoper0".to_string(), Uint128::from(33u128)),
            ("valoper1".to_string(), Uint128::from(33u128)),
            ("valoper2".to_string(), Uint128::from(34u128))
        ]
    );
}

#[test]
fn test_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("not_owner", &[]),
        drop_staking_base::msg::strategy::ExecuteMsg::UpdateConfig {
            new_config: drop_staking_base::msg::strategy::ConfigOptional {
                factory_contract: Some("new_factory_contract".to_string()),
                denom: Some("new_denom".to_string()),
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        crate::error::ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);

    FACTORY_CONTRACT
        .save(
            deps.as_mut().storage,
            &cosmwasm_std::Addr::unchecked(FACTORY_CONTRACT_ADDR.to_string()),
        )
        .unwrap();
    DENOM
        .save(deps.as_mut().storage, &"denom".to_string())
        .unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let res = execute(
        deps_mut.into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::strategy::ExecuteMsg::UpdateConfig {
            new_config: drop_staking_base::msg::strategy::ConfigOptional {
                factory_contract: Some("new_factory_contract".to_string()),
                denom: Some("new_denom".to_string()),
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_event(
            cosmwasm_std::Event::new(
                "crates.io:drop-staking__drop-strategy-config_update".to_string()
            )
            .add_attributes(vec![
                cosmwasm_std::attr(
                    "factory_contract".to_string(),
                    "new_factory_contract".to_string()
                ),
                cosmwasm_std::attr("denom".to_string(), "new_denom".to_string())
            ])
        )
    )
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::strategy::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: Some(cw_ownable::Expiration::Never {}),
            },
        ),
    )
    .unwrap();
    execute(
        deps.as_mut().into_empty(),
        mock_env(),
        mock_info("new_owner", &[]),
        drop_staking_base::msg::strategy::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::AcceptOwnership {},
        ),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        query(
            deps.as_ref().into_empty(),
            mock_env(),
            drop_staking_base::msg::strategy::QueryMsg::Ownership {},
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
