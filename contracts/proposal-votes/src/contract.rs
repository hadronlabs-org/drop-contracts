use std::collections::HashSet;

use cosmwasm_std::{
    attr, ensure_eq, entry_point, to_json_binary, Addr, Attribute, Deps, Order, SubMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::response;
use lido_staking_base::error::proposal_votes::{ContractError, ContractResult};
use lido_staking_base::msg::proposal_votes::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use lido_staking_base::state::proposal_votes::{
    Config, ConfigResponse, ACTIVE_PROPOSALS, CORE_ADDRESS, GOV_HELPER_ADDRESS, VOTERS,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::interchain_queries::v045::register_queries::new_register_gov_proposal_votes_query_msg;

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let core = deps.api.addr_validate(&msg.core_address)?;
    let gov_helper = deps.api.addr_validate(&msg.gov_helper_address)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core.as_ref()))?;

    CORE_ADDRESS.save(deps.storage, &core)?;
    GOV_HELPER_ADDRESS.save(deps.storage, &gov_helper)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("core_address", msg.core_address),
            attr("gov_helper_address", msg.gov_helper_address),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();
    let gov_helper_address = GOV_HELPER_ADDRESS.load(deps.storage)?.into_string();

    Ok(to_json_binary(&ConfigResponse {
        core_address,
        gov_helper_address,
    })?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateConfig {
            core_address,
            gov_helper_address,
        } => execute_update_config(deps, info, core_address, gov_helper_address),
        ExecuteMsg::UpdateActiveProposals { active_proposals } => {
            execute_update_active_proposals(deps, info, active_proposals)
        }
        ExecuteMsg::UpdateVotersList { voters } => execute_update_voters_list(voters),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    core_address: Option<String>,
    gov_helper_address: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        CORE_ADDRESS.save(deps.storage, &core_address)?;
        attrs.push(attr("core_address", core_address))
    }

    if let Some(gov_helper_address) = gov_helper_address {
        let gov_helper_address = deps.api.addr_validate(&gov_helper_address)?;
        GOV_HELPER_ADDRESS.save(deps.storage, &gov_helper_address)?;
        attrs.push(attr("gov_helper_address", gov_helper_address))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn execute_update_voters_list(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    voters: Vec<Addr>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    VOTERS.save(deps.storage, &voters)?;

    Ok(response(
        "config_update",
        CONTRACT_NAME,
        [attr("total_count", voters.len().to_string())],
    ))
}

fn execute_update_active_proposals(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    active_proposals: Vec<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let gov_helper_address = GOV_HELPER_ADDRESS.load(deps.storage)?.into_string();

    ensure_eq!(
        gov_helper_address,
        info.sender,
        ContractError::Unauthorized {}
    );

    let old_active_proposals = ACTIVE_PROPOSALS.load(deps.storage)?;

    let active_proposals_set: HashSet<_> = active_proposals.into_iter().collect();
    let old_active_proposals_set: HashSet<_> = old_active_proposals.into_iter().collect();

    let new_proposals: HashSet<_> = active_proposals_set
        .difference(&old_active_proposals_set)
        .cloned()
        .collect();
    let proposals_to_remove: HashSet<_> = old_active_proposals_set
        .difference(&active_proposals_set)
        .cloned()
        .collect();

    if new_proposals.is_empty() && proposals_to_remove.is_empty() {
        return Ok(response(
            "update_active_proposals",
            CONTRACT_NAME,
            [attr("total_count", active_proposals.len().to_string())],
        ));
    }

    let msg = new_register_gov_proposal_votes_query_msg(
        config.connection_id.clone(),
        validators,
        config.profile_update_period,
    )?;

    let sub_msg = SubMsg::reply_on_success(msg, VALIDATOR_PROFILE_REPLY_ID);

    Ok(Response::new().add_submessage(sub_msg))
}

fn execute_update_validators(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<ValidatorData>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let total_count = validators.len();

    // TODO: implement notification of the validator stats contract about new validators set
    VALIDATORS_SET.clear(deps.storage);

    for validator in validators {
        let valoper_address = validator.valoper_address.clone();

        VALIDATORS_SET.save(
            deps.storage,
            valoper_address,
            &ValidatorInfo {
                valoper_address: validator.valoper_address,
                weight: validator.weight,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Default::default(),
                tombstone: false,
                jailed_number: None,
            },
        )?;
    }

    Ok(response(
        "update_validators",
        CONTRACT_NAME,
        [attr("total_count", total_count.to_string())],
    ))
}

fn execute_update_validators_info(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators_update: Vec<ValidatorInfoUpdate>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        config.stats_contract,
        info.sender,
        ContractError::Unauthorized {}
    );

    let total_count = validators_update.len();

    for update in validators_update {
        // TODO: Implement logic to modify validator set based in incoming validator info
        let validator =
            VALIDATORS_SET.may_load(deps.storage, update.valoper_address.to_string())?;
        if validator.is_none() {
            continue;
        }
        let mut validator = validator.unwrap();

        if update.last_commission_in_range.is_some() {
            validator.last_commission_in_range = update.last_commission_in_range;
        }
        if update.last_processed_local_height.is_some() {
            validator.last_processed_local_height = update.last_processed_local_height;
        }
        if update.last_processed_remote_height.is_some() {
            validator.last_processed_remote_height = update.last_processed_remote_height;
        }
        if update.last_validated_height.is_some() {
            validator.last_validated_height = update.last_validated_height;
        }
        if update.jailed_number.is_some() {
            validator.jailed_number = update.jailed_number;
        }

        validator.uptime = update.uptime;
        validator.tombstone = update.tombstone;

        VALIDATORS_SET.save(deps.storage, validator.valoper_address.clone(), &validator)?;
    }

    Ok(response(
        "update_validators_info",
        CONTRACT_NAME,
        [attr("total_count", total_count.to_string())],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
