use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{attr, ensure_eq, to_json_binary, Attribute, Deps, Uint128};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use drop_helpers::answer::response;
use drop_staking_base::msg::strategy::{
    Config, ConfigOptional, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::strategy::{
    DENOM, DISTRIBUTION_ADDRESS, PUPPETEER_ADDRESS, VALIDATOR_SET_ADDRESS,
};
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

    let puppeteer = deps.api.addr_validate(&msg.puppeteer_address)?;
    PUPPETEER_ADDRESS.save(deps.storage, &puppeteer)?;

    let validator_set = deps.api.addr_validate(&msg.validator_set_address)?;
    VALIDATOR_SET_ADDRESS.save(deps.storage, &validator_set)?;

    let distribution = deps.api.addr_validate(&msg.distribution_address)?;
    DISTRIBUTION_ADDRESS.save(deps.storage, &distribution)?;

    DENOM.save(deps.storage, &msg.denom)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("owner", msg.owner),
            attr("puppeteer_address", msg.puppeteer_address),
            attr("validator_set_address", msg.validator_set_address),
            attr("distribution_address", msg.distribution_address),
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
    let puppeteer_address = PUPPETEER_ADDRESS.load(deps.storage)?.into_string();
    let validator_set_address = VALIDATOR_SET_ADDRESS.load(deps.storage)?.into_string();
    let distribution_address = DISTRIBUTION_ADDRESS.load(deps.storage)?.into_string();
    let denom = DENOM.load(deps.storage)?;

    Ok(to_json_binary(&Config {
        puppeteer_address,
        validator_set_address,
        distribution_address,
        denom,
    })?)
}

pub fn query_calc_deposit(deps: Deps, deposit: Uint128) -> ContractResult<Binary> {
    let distribution_address = DISTRIBUTION_ADDRESS.load(deps.storage)?.into_string();

    let delegations = prepare_delegation_data(deps)?;

    let deposit_changes: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        distribution_address,
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
    let distribution_address = DISTRIBUTION_ADDRESS.load(deps.storage)?.into_string();

    let delegations = prepare_delegation_data(deps)?;

    let deposit_changes: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        distribution_address,
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
    let puppeteer_address = PUPPETEER_ADDRESS.load(deps.storage)?.into_string();
    let validator_set_address = VALIDATOR_SET_ADDRESS.load(deps.storage)?.into_string();
    let denom = DENOM.load(deps.storage)?;
    let account_delegations: drop_staking_base::msg::puppeteer::DelegationsResponse =
        deps.querier.query_wasm_smart(
            puppeteer_address,
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

    let validator_set: Vec<drop_staking_base::state::validatorset::ValidatorInfo> =
        deps.querier.query_wasm_smart(
            validator_set_address,
            &drop_staking_base::msg::validatorset::QueryMsg::Validators {},
        )?;

    let mut delegations: Vec<drop_staking_base::msg::distribution::Delegation> = Vec::new();
    let mut total_delegations: Uint128 = Uint128::zero();
    let mut total_weight: u64 = 0;
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
        };

        total_delegations += validator_denom_delegation;
        total_weight += validator.weight;
        delegations.push(delegation);
    }

    Ok(drop_staking_base::msg::distribution::Delegations {
        total: total_delegations,
        total_weight,
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

    if let Some(puppeteer_address) = new_config.puppeteer_address {
        let puppeteer_address = deps.api.addr_validate(&puppeteer_address)?;
        PUPPETEER_ADDRESS.save(deps.storage, &puppeteer_address)?;
        attrs.push(attr("puppeteer_address", puppeteer_address))
    }

    if let Some(validator_set_address) = new_config.validator_set_address {
        let validator_set_address = deps.api.addr_validate(&validator_set_address)?;
        VALIDATOR_SET_ADDRESS.save(deps.storage, &validator_set_address)?;
        attrs.push(attr("validator_set_address", validator_set_address))
    }

    if let Some(distribution_address) = new_config.distribution_address {
        let distribution_address = deps.api.addr_validate(&distribution_address)?;
        DISTRIBUTION_ADDRESS.save(deps.storage, &distribution_address)?;
        attrs.push(attr("distribution_address", distribution_address))
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
