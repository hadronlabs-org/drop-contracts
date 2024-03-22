use std::collections::HashMap;

use cosmwasm_std::{attr, entry_point, to_json_binary, Attribute, Deps, Uint128};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use drop_helpers::answer::response;
use drop_staking_base::msg::strategy::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::strategy::{
    CORE_ADDRESS, DENOM, DISTRIBUTION_ADDRESS, PUPPETEER_ADDRESS, VALIDATOR_SET_ADDRESS,
};
use neutron_sdk::NeutronResult;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let core_address = deps.api.addr_validate(&msg.core_address)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core_address.as_ref()))?;
    CORE_ADDRESS.save(deps.storage, &core_address)?;

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
            attr("core_address", msg.core_address),
            attr("puppeteer_address", msg.puppeteer_address),
            attr("validator_set_address", msg.validator_set_address),
            attr("distribution_address", msg.distribution_address),
            attr("denom", msg.denom),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::CalcDeposit { deposit } => query_calc_deposit(deps, deposit),
        QueryMsg::CalcWithdraw { withdraw } => query_calc_withdraw(deps, withdraw),
    }
}

fn query_config(deps: Deps, _env: Env) -> NeutronResult<Binary> {
    let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();
    let puppeteer_address = PUPPETEER_ADDRESS.load(deps.storage)?.into_string();
    let validator_set_address = VALIDATOR_SET_ADDRESS.load(deps.storage)?.into_string();
    let distribution_address = DISTRIBUTION_ADDRESS.load(deps.storage)?.into_string();
    let denom = DENOM.load(deps.storage)?;

    Ok(to_json_binary(&ConfigResponse {
        core_address,
        puppeteer_address,
        validator_set_address,
        distribution_address,
        denom,
    })?)
}

pub fn query_calc_deposit(deps: Deps, deposit: Uint128) -> NeutronResult<Binary> {
    let distribution_address = DISTRIBUTION_ADDRESS.load(deps.storage)?.into_string();

    let delegations: Vec<drop_staking_base::msg::distribution::Delegation> =
        prepare_delegation_data(deps)?;

    let ideal_deposit: Vec<drop_staking_base::msg::distribution::IdealDelegation> =
        deps.querier.query_wasm_smart(
            distribution_address,
            &drop_staking_base::msg::distribution::QueryMsg::CalcDeposit {
                deposit,
                delegations,
            },
        )?;

    Ok(to_json_binary(&ideal_deposit)?)
}

pub fn query_calc_withdraw(deps: Deps, withdraw: Uint128) -> NeutronResult<Binary> {
    let distribution_address = DISTRIBUTION_ADDRESS.load(deps.storage)?.into_string();

    let delegations: Vec<drop_staking_base::msg::distribution::Delegation> =
        prepare_delegation_data(deps)?;

    let ideal_deposit: Vec<drop_staking_base::msg::distribution::IdealDelegation> =
        deps.querier.query_wasm_smart(
            distribution_address,
            &drop_staking_base::msg::distribution::QueryMsg::CalcWithdraw {
                withdraw,
                delegations,
            },
        )?;

    Ok(to_json_binary(&ideal_deposit)?)
}

fn prepare_delegation_data(
    deps: Deps,
) -> NeutronResult<Vec<drop_staking_base::msg::distribution::Delegation>> {
    let puppeteer_address = PUPPETEER_ADDRESS.load(deps.storage)?.into_string();
    let validator_set_address = VALIDATOR_SET_ADDRESS.load(deps.storage)?.into_string();
    let denom = DENOM.load(deps.storage)?;
    let account_delegations: drop_staking_base::msg::puppeteer::DelegationsResponse =
        deps.querier.query_wasm_smart(
            puppeteer_address,
            &drop_puppeteer_base::msg::QueryMsg::Extention {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

    let validator_set: Vec<drop_staking_base::state::validatorset::ValidatorInfo> =
        deps.querier.query_wasm_smart(
            validator_set_address,
            &drop_staking_base::msg::validatorset::QueryMsg::Validators {},
        )?;

    let mut delegations: Vec<drop_staking_base::msg::distribution::Delegation> = Vec::new();
    let delegation_validator_map: HashMap<_, _> = account_delegations
        .0
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

        delegations.push(delegation);
    }

    Ok(delegations)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            core_address,
            puppeteer_address,
            validator_set_address,
            distribution_address,
            denom,
        } => exec_config_update(
            deps,
            core_address,
            puppeteer_address,
            validator_set_address,
            distribution_address,
            denom,
        ),
    }
}

fn exec_config_update(
    deps: DepsMut,
    core_address: Option<String>,
    puppeteer_address: Option<String>,
    validator_set_address: Option<String>,
    distribution_address: Option<String>,
    denom: Option<String>,
) -> NeutronResult<Response> {
    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        CORE_ADDRESS.save(deps.storage, &core_address)?;
        attrs.push(attr("core_address", core_address))
    }

    if let Some(puppeteer_address) = puppeteer_address {
        let puppeteer_address = deps.api.addr_validate(&puppeteer_address)?;
        PUPPETEER_ADDRESS.save(deps.storage, &puppeteer_address)?;
        attrs.push(attr("puppeteer_address", puppeteer_address))
    }

    if let Some(validator_set_address) = validator_set_address {
        let validator_set_address = deps.api.addr_validate(&validator_set_address)?;
        VALIDATOR_SET_ADDRESS.save(deps.storage, &validator_set_address)?;
        attrs.push(attr("validator_set_address", validator_set_address))
    }

    if let Some(distribution_address) = distribution_address {
        let distribution_address = deps.api.addr_validate(&distribution_address)?;
        DISTRIBUTION_ADDRESS.save(deps.storage, &distribution_address)?;
        attrs.push(attr("distribution_address", distribution_address))
    }

    if let Some(denom) = denom {
        DENOM.save(deps.storage, &denom)?;
        attrs.push(attr("denom", denom))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
