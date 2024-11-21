use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{attr, ensure_eq, to_json_binary, Attribute, Deps, Uint128};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use drop_helpers::answer::response;
use drop_staking_base::msg::strategy::{
    Config, ConfigOptional, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::strategy::{DENOM, FACTORY_CONTRACT};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};
use std::collections::HashMap;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let factory_contract = deps.api.addr_validate(&msg.factory_contract)?;
    FACTORY_CONTRACT.save(deps.storage, &factory_contract)?;

    DENOM.save(deps.storage, &msg.denom)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("owner", msg.owner),
            attr("factory_contract", msg.factory_contract),
            attr("denom", msg.denom),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::CalcDeposit { deposit } => query_calc_deposit(deps, deposit),
        QueryMsg::CalcWithdraw { withdraw } => query_calc_withdraw(deps, withdraw),
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
    }
}

fn query_config(deps: Deps, _env: Env) -> ContractResult<Binary> {
    let factory_contract = FACTORY_CONTRACT.load(deps.storage)?.into_string();
    let denom = DENOM.load(deps.storage)?;

    Ok(to_json_binary(&Config {
        factory_contract,
        denom,
    })?)
}

pub fn query_calc_deposit(deps: Deps, deposit: Uint128) -> ContractResult<Binary> {
    let factory_contract = FACTORY_CONTRACT.load(deps.storage)?.to_string();
    println!("factory_contract: {:?}", factory_contract);
    let addrs = drop_helpers::get_contracts!(deps, factory_contract, distribution_contract);
    println!("addrs: {:?}", addrs);
    let delegations = prepare_delegation_data(deps)?;

    let deposit_changes: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        addrs.distribution_contract.to_string(),
        &drop_staking_base::msg::distribution::QueryMsg::CalcDeposit {
            deposit,
            delegations,
        },
    )?;

    let total_deposit_changes: Uint128 = deposit_changes.iter().map(|(_, amount)| amount).sum();
    ensure_eq!(
        total_deposit_changes,
        deposit,
        ContractError::WrongDepositAndCalculation {}
    );

    Ok(to_json_binary(&deposit_changes)?)
}

pub fn query_calc_withdraw(deps: Deps, withdraw: Uint128) -> ContractResult<Binary> {
    let factory_contract = FACTORY_CONTRACT.load(deps.storage)?.to_string();
    let addrs = drop_helpers::get_contracts!(deps, factory_contract, distribution_contract);

    let delegations = prepare_delegation_data(deps)?;

    let deposit_changes: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        addrs.distribution_contract.to_string(),
        &drop_staking_base::msg::distribution::QueryMsg::CalcWithdraw {
            withdraw,
            delegations,
        },
    )?;

    let total_deposit_changes: Uint128 = deposit_changes.iter().map(|(_, amount)| amount).sum();
    ensure_eq!(
        total_deposit_changes,
        withdraw,
        ContractError::WrongWithdrawAndCalculation {}
    );

    Ok(to_json_binary(&deposit_changes)?)
}

fn prepare_delegation_data(
    deps: Deps,
) -> NeutronResult<drop_staking_base::msg::distribution::Delegations> {
    let factory_contract = FACTORY_CONTRACT.load(deps.storage)?.to_string();
    let addrs = drop_helpers::get_contracts!(
        deps,
        factory_contract,
        puppeteer_contract,
        validator_set_contract
    );
    let denom = DENOM.load(deps.storage)?;
    let account_delegations: drop_staking_base::msg::puppeteer::DelegationsResponse =
        deps.querier.query_wasm_smart(
            addrs.puppeteer_contract.to_string(),
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

    let validator_set: Vec<drop_staking_base::state::validatorset::ValidatorInfo> =
        deps.querier.query_wasm_smart(
            addrs.validator_set_contract.to_string(),
            &drop_staking_base::msg::validatorset::QueryMsg::Validators {},
        )?;

    let mut delegations: Vec<drop_staking_base::msg::distribution::Delegation> = Vec::new();
    let mut total_delegations: Uint128 = Uint128::zero();
    let mut total_weight: u64 = 0;
    let mut total_on_top = Uint128::zero();
    let delegation_validator_map: HashMap<_, _> = account_delegations
        .delegations
        .delegations
        .iter()
        .filter(|delegation| delegation.amount.denom == denom)
        .map(|delegation| (delegation.validator.clone(), delegation.amount.amount))
        .collect();

    for validator in validator_set.iter() {
        let validator_denom_delegation = delegation_validator_map
            .get(&validator.valoper_address)
            .copied()
            .unwrap_or_default();

        let delegation = drop_staking_base::msg::distribution::Delegation {
            valoper_address: validator.valoper_address.clone(),
            stake: validator_denom_delegation,
            weight: validator.weight,
            on_top: validator.on_top,
        };

        total_delegations += validator_denom_delegation;
        total_weight += validator.weight;
        total_on_top += validator.on_top;
        delegations.push(delegation);
    }

    Ok(drop_staking_base::msg::distribution::Delegations {
        total_stake: total_delegations,
        total_weight,
        total_on_top,
        delegations,
    })
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::UpdateConfig { new_config } => exec_config_update(deps, info, new_config),
    }
}

fn exec_config_update(
    deps: DepsMut,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(factory_contract) = new_config.factory_contract {
        let factory_contract = deps.api.addr_validate(&factory_contract)?;
        FACTORY_CONTRACT.save(deps.storage, &factory_contract)?;
        attrs.push(attr("factory_contract", factory_contract))
    }

    if let Some(denom) = new_config.denom {
        DENOM.save(deps.storage, &denom)?;
        attrs.push(attr("denom", denom))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}
