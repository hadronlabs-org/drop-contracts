use std::collections::HashSet;

use cosmwasm_std::{
    attr, ensure_eq, entry_point, to_json_binary, Addr, Attribute, Deps, Order, Reply, SubMsg,
    SubMsgResult,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::response;
use lido_helpers::reply::get_query_id;
use lido_staking_base::error::proposal_votes::{ContractError, ContractResult};
use lido_staking_base::msg::proposal_votes::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use lido_staking_base::state::proposal_votes::{
    Config, ACTIVE_PROPOSALS, CONFIG, PROPOSALS_VOTES_REMOVE_REPLY_ID, PROPOSALS_VOTES_REPLY_ID,
    QUERY_ID, VOTERS,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::{NeutronQuery, QueryRegisteredQueryResultResponse};
use neutron_sdk::interchain_queries::queries::get_raw_interchain_query_result;
use neutron_sdk::interchain_queries::types::KVReconstruct;
use neutron_sdk::interchain_queries::v045::register_queries::new_register_gov_proposal_votes_query_msg;
use neutron_sdk::interchain_queries::v045::types::GovernmentProposalVotes;
use neutron_sdk::sudo::msg::SudoMsg;

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

    let config = &Config {
        connection_id: msg.connection_id,
        port_id: msg.port_id,
        update_period: msg.update_period,
        core_address: msg.core_address,
        gov_helper_address: gov_helper.to_string(),
    };

    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("connection_id", msg.core_address),
            attr("port_id", msg.port_id),
            attr("update_period", msg.update_period.to_string()),
            attr("core_address", msg.core_address),
            attr("gov_helper_address", msg.gov_helper_address),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
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
            connection_id,
            port_id,
            update_period,
            core_address,
            gov_helper_address,
        } => execute_update_config(
            deps,
            info,
            connection_id,
            port_id,
            update_period,
            core_address,
            gov_helper_address,
        ),
        ExecuteMsg::UpdateActiveProposals { active_proposals } => {
            execute_update_active_proposals(deps, info, active_proposals)
        }
        ExecuteMsg::UpdateVotersList { voters } => execute_update_voters_list(deps, info, voters),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    connection_id: Option<String>,
    port_id: Option<String>,
    update_period: Option<u64>,
    core_address: Option<String>,
    gov_helper_address: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        config.core_address = core_address.to_string();
        attrs.push(attr("core_address", core_address))
    }

    if let Some(gov_helper_address) = gov_helper_address {
        let gov_helper_address = deps.api.addr_validate(&gov_helper_address)?;
        config.gov_helper_address = gov_helper_address.to_string();
        attrs.push(attr("gov_helper_address", gov_helper_address))
    }

    if let Some(connection_id) = connection_id {
        config.connection_id = connection_id;
        attrs.push(attr("connection_id", connection_id))
    }

    if let Some(port_id) = port_id {
        config.port_id = port_id;
        attrs.push(attr("port_id", port_id))
    }

    if let Some(update_period) = update_period {
        config.update_period = update_period;
        attrs.push(attr("update_period", update_period.to_string()))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn execute_update_voters_list(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    voters: Vec<String>,
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
    let config = CONFIG.load(deps.storage)?;

    ensure_eq!(
        config.gov_helper_address,
        info.sender,
        ContractError::Unauthorized {}
    );

    let query_id = QUERY_ID.may_load(deps.storage)?;

    if active_proposals.is_empty() && query_id.is_some() {
        let query_id = query_id.unwrap();
        let msg = NeutronMsg::remove_interchain_query(query_id);
        let sub_msg = SubMsg::reply_on_success(msg, PROPOSALS_VOTES_REMOVE_REPLY_ID);

        return Ok(response(
            "update_active_proposals",
            CONTRACT_NAME,
            [attr("remove_query", "true")],
        )
        .add_submessage(sub_msg));
    }

    let voters = VOTERS.may_load(deps.storage)?;

    if !active_proposals.is_empty() && query_id.is_none() {
        if let Some(voters) = voters {
            let msg = new_register_gov_proposal_votes_query_msg(
                config.connection_id.clone(),
                active_proposals,
                voters,
                config.update_period,
            )?;

            let sub_msg = SubMsg::reply_on_success(msg, PROPOSALS_VOTES_REPLY_ID);

            return Ok(response(
                "update_active_proposals",
                CONTRACT_NAME,
                [attr("remove_query", "true")],
            )
            .add_submessage(sub_msg));
        }
    }

    let old_active_proposals = ACTIVE_PROPOSALS
        .may_load(deps.storage)?
        .unwrap_or_else(|| vec![]);

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

    if !new_proposals.is_empty() || !proposals_to_remove.is_empty() {
        let query_id = query_id.unwrap();
        let msg = NeutronMsg::update_interchain_query(query_id);
        let sub_msg = SubMsg::reply_on_success(msg, PROPOSALS_VOTES_REMOVE_REPLY_ID);

        return Ok(response(
            "update_active_proposals",
            CONTRACT_NAME,
            [attr("remove_query", "true")],
        )
        .add_submessage(sub_msg));

        return Ok(response(
            "update_active_proposals",
            CONTRACT_NAME,
            [attr("total_count", active_proposals.len().to_string())],
        ));
    }

    Ok(Response::new().add_submessage(sub_msg))
}

#[entry_point]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: SudoMsg,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?},  block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, env, query_id),
        _ => Ok(Response::default()),
    }
}

pub fn sudo_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_kv_query_result call: {query_id:?}",
    ));

    let votes_query_id = QUERY_ID.may_load(deps.storage)?;

    deps.api.debug(&format!(
        "WASMDEBUG: sudo_kv_query_result proposal_votes_query_id: {:?}",
        query_id.clone()
    ));

    let optional_query_id = Some(query_id);

    let interchain_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;

    if optional_query_id == votes_query_id {
        return sudo_proposal_votes(deps, _env, interchain_query_result);
    } else {
        deps.api.debug(&format!(
            "WASMDEBUG: sudo_kv_query_result query_id: {:?}",
            query_id
        ));
    }

    Ok(Response::default())
}

fn sudo_proposal_votes(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    interchain_query_result: QueryRegisteredQueryResultResponse,
) -> ContractResult<Response<NeutronMsg>> {
    let data: GovernmentProposalVotes =
        KVReconstruct::reconstruct(&interchain_query_result.result.kv_results)?;

    deps.api
        .debug(&format!("WASMDEBUG: validator_info_sudo data: {data:?}",));

    Ok(Response::new())
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: reply msg: {msg:?}").as_str());

    match msg.id {
        PROPOSALS_VOTES_REPLY_ID => proposals_votes_reply(deps, env, msg),
        PROPOSALS_VOTES_REMOVE_REPLY_ID => proposals_votes_remove_reply(deps, env, msg),
    }
}

fn proposals_votes_reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(&format!("WASMDEBUG: proposals_votes_reply call: {msg:?}",));

    let query_id = get_query_id(msg.result)?;

    QUERY_ID.save(deps.storage, &query_id)?;

    Ok(Response::new())
}

fn proposals_votes_remove_reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    deps.api.debug(&format!(
        "WASMDEBUG: proposals_votes_remove_reply call: {msg:?}",
    ));

    let query_id = get_query_id(msg.result)?;

    QUERY_ID.remove(deps.storage);

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
