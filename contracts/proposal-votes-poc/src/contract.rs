use std::collections::HashSet;

use cosmwasm_std::{
    attr, ensure_eq, entry_point, to_json_binary, Attribute, CosmosMsg, Deps, Reply, SubMsg,
    WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use drop_helpers::answer::response;
use drop_helpers::query_id::get_query_id;
use drop_staking_base::msg::proposal_votes::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use drop_staking_base::msg::provider_proposals::ExecuteMsg as ProviderProposalsExecuteMsg;
use drop_staking_base::state::proposal_votes::{
    Config, ConfigOptional, Metrics, ACTIVE_PROPOSALS, CONFIG, PROPOSALS_VOTES_REMOVE_REPLY_ID,
    PROPOSALS_VOTES_REPLY_ID, QUERY_ID, VOTERS,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::{NeutronQuery, QueryRegisteredQueryResultResponse};
use neutron_sdk::interchain_queries::queries::get_raw_interchain_query_result;
use neutron_sdk::interchain_queries::types::KVReconstruct;
use neutron_sdk::interchain_queries::v045::types::GovernmentProposalVotes;
use neutron_sdk::interchain_queries::v047::register_queries::{
    new_register_gov_proposals_voters_votes_query_msg, update_gov_proposal_votes_query_msg,
};
use neutron_sdk::sudo::msg::SudoMsg;

use crate::error::{ContractError, ContractResult};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));

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
    let provider_proposals = deps.api.addr_validate(&msg.provider_proposals_address)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core.as_ref()))?;

    let config = &Config {
        connection_id: msg.connection_id.clone(),
        port_id: msg.port_id.clone(),
        update_period: msg.update_period,
        core_address: msg.core_address.to_string(),
        provider_proposals_address: provider_proposals.to_string(),
    };

    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("connection_id", msg.connection_id),
            attr("port_id", msg.port_id),
            attr("update_period", msg.update_period.to_string()),
            attr("core_address", msg.core_address),
            attr("provider_proposals_address", msg.provider_proposals_address),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Metrics {} => query_metrics(deps),
    }
}

fn query_config(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

fn query_metrics(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let voters = VOTERS.may_load(deps.storage)?.unwrap_or_default();

    to_json_binary(&Metrics {
        total_voters: voters.len() as u64,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::UpdateActiveProposals { active_proposals } => {
            execute_update_active_proposals(deps, info, active_proposals)
        }
        ExecuteMsg::UpdateVotersList { voters } => execute_update_voters_list(deps, info, voters),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = new_config.core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        config.core_address = core_address.to_string();
        attrs.push(attr("core_address", core_address))
    }

    if let Some(provider_proposals_address) = new_config.provider_proposals_address {
        let provider_proposals_address = deps.api.addr_validate(&provider_proposals_address)?;
        config.provider_proposals_address = provider_proposals_address.to_string();
        attrs.push(attr(
            "provider_proposals_address",
            provider_proposals_address,
        ))
    }

    if let Some(connection_id) = new_config.connection_id {
        config.connection_id = connection_id.clone();
        attrs.push(attr("connection_id", connection_id))
    }

    if let Some(port_id) = new_config.port_id {
        config.port_id = port_id.clone();
        attrs.push(attr("port_id", port_id))
    }

    if let Some(update_period) = new_config.update_period {
        config.update_period = update_period;
        attrs.push(attr("update_period", update_period.to_string()))
    }

    CONFIG.save(deps.storage, &config)?;

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
        config.provider_proposals_address,
        info.sender,
        ContractError::Unauthorized {}
    );

    let query_id = QUERY_ID.may_load(deps.storage)?;

    if active_proposals.is_empty() && query_id.is_none() {
        return Ok(Response::default());
    }

    if active_proposals.is_empty() && query_id.is_some() {
        if let Some(query_id) = query_id {
            return remove_votes_interchain_query(query_id);
        }
    }

    process_new_data(deps, &config, query_id, active_proposals)
}

fn process_new_data(
    deps: DepsMut<NeutronQuery>,
    config: &Config,
    query_id: Option<u64>,
    active_proposals: Vec<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let voters = VOTERS.may_load(deps.storage)?;

    let mut sub_msgs: Vec<SubMsg<NeutronMsg>> = Vec::new();
    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(voters) = voters {
        attrs.push(attr("total_proposals", active_proposals.len().to_string()));
        attrs.push(attr("total_voters", voters.len().to_string()));

        if !active_proposals.is_empty() && query_id.is_none() {
            ACTIVE_PROPOSALS.save(deps.storage, &active_proposals)?;

            sub_msgs.push(register_votes_interchain_query(
                config,
                active_proposals.to_owned(),
                voters.to_owned(),
            )?);
        }

        let old_active_proposals = ACTIVE_PROPOSALS.may_load(deps.storage)?.unwrap_or_default();

        let active_proposals_set: HashSet<_> = active_proposals.clone().into_iter().collect();
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
            if let Some(query_id) = query_id {
                ACTIVE_PROPOSALS.save(deps.storage, &active_proposals)?;

                sub_msgs.push(update_votes_interchain_query(
                    query_id,
                    active_proposals,
                    voters,
                )?);
            }
        }
    }

    Ok(response("update_votes_interchain_query", CONTRACT_NAME, attrs).add_submessages(sub_msgs))
}

fn update_votes_interchain_query(
    query_id: u64,
    active_proposals: Vec<u64>,
    voters: Vec<String>,
) -> ContractResult<SubMsg<NeutronMsg>> {
    let msg = update_gov_proposal_votes_query_msg(
        query_id,
        active_proposals.to_owned(),
        voters.to_owned(),
        None,
    )?;

    Ok(SubMsg::reply_on_success(msg, PROPOSALS_VOTES_REPLY_ID))
}

fn register_votes_interchain_query(
    config: &Config,
    active_proposals: Vec<u64>,
    voters: Vec<String>,
) -> ContractResult<SubMsg<NeutronMsg>> {
    let msg = new_register_gov_proposals_voters_votes_query_msg(
        config.connection_id.to_string(),
        active_proposals,
        voters,
        config.update_period,
    )?;

    Ok(SubMsg::reply_on_success(msg, PROPOSALS_VOTES_REPLY_ID))
}

fn remove_votes_interchain_query(query_id: u64) -> ContractResult<Response<NeutronMsg>> {
    let msg = NeutronMsg::remove_interchain_query(query_id);
    let sub_msg = SubMsg::reply_on_success(msg, PROPOSALS_VOTES_REMOVE_REPLY_ID);

    Ok(response(
        "remove_votes_interchain_query",
        CONTRACT_NAME,
        [attr("query_id", query_id.to_string())],
    )
    .add_submessage(sub_msg))
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

    let interchain_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;

    if Some(query_id) == votes_query_id {
        return sudo_proposal_votes(deps, interchain_query_result);
    }

    Ok(Response::default())
}

fn sudo_proposal_votes(
    deps: DepsMut<NeutronQuery>,
    interchain_query_result: QueryRegisteredQueryResultResponse,
) -> ContractResult<Response<NeutronMsg>> {
    let data: GovernmentProposalVotes =
        KVReconstruct::reconstruct(&interchain_query_result.result.kv_results)?;

    deps.api
        .debug(&format!("WASMDEBUG: sudo_proposal_votes data: {data:?}",));

    let config = CONFIG.load(deps.storage)?;

    let msg: CosmosMsg<NeutronMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.provider_proposals_address,
        msg: to_json_binary(&ProviderProposalsExecuteMsg::UpdateProposalVotes {
            votes: data.proposal_votes,
        })?,
        funds: vec![],
    });

    Ok(Response::new().add_message(msg))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: reply msg: {msg:?}").as_str());

    match msg.id {
        PROPOSALS_VOTES_REPLY_ID => proposals_votes_reply(deps, env, msg),
        PROPOSALS_VOTES_REMOVE_REPLY_ID => proposals_votes_remove_reply(deps, env, msg),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn proposals_votes_reply(deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    let query_id = get_query_id(msg.result)?;

    QUERY_ID.save(deps.storage, &query_id)?;

    Ok(Response::new())
}

fn proposals_votes_remove_reply(deps: DepsMut, _env: Env, _msg: Reply) -> ContractResult<Response> {
    QUERY_ID.remove(deps.storage);

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
