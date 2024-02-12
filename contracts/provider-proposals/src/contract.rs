use cosmwasm_std::{attr, ensure_eq, entry_point, to_json_binary, Attribute, Deps, Reply};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::response;
use lido_helpers::reply::get_query_id;
use lido_staking_base::msg::provider_proposals::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use lido_staking_base::state::provider_proposals::{Config, CONFIG, QUERY_ID};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::{NeutronQuery, QueryRegisteredQueryResultResponse};
use neutron_sdk::interchain_queries::queries::get_raw_interchain_query_result;
use neutron_sdk::interchain_queries::types::KVReconstruct;
use neutron_sdk::interchain_queries::v045::types::{GovernmentProposalVotes, ProposalVote};
use neutron_sdk::sudo::msg::SudoMsg;

use crate::error::ContractResult;

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
    let proposal_votes_ = deps.api.addr_validate(&msg.proposal_votes_address)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core.as_ref()))?;

    let config = &Config {
        connection_id: msg.connection_id.clone(),
        port_id: msg.port_id.clone(),
        update_period: msg.update_period,
        core_address: msg.core_address.to_string(),
        proposal_votes_address: proposal_votes_.to_string(),
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
            attr("proposal_votes_address", msg.proposal_votes_address),
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
            proposal_votes_address,
        } => execute_update_config(
            deps,
            info,
            connection_id,
            port_id,
            update_period,
            core_address,
            proposal_votes_address,
        ),
        ExecuteMsg::UpdateProposalVotes { votes } => execute_update_votes(deps, info, votes),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    connection_id: Option<String>,
    port_id: Option<String>,
    update_period: Option<u64>,
    core_address: Option<String>,
    proposal_votes_address: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        config.core_address = core_address.to_string();
        attrs.push(attr("core_address", core_address))
    }

    if let Some(proposal_votes_address) = proposal_votes_address {
        let proposal_votes_address = deps.api.addr_validate(&proposal_votes_address)?;
        config.proposal_votes_address = proposal_votes_address.to_string();
        attrs.push(attr("proposal_votes_address", proposal_votes_address))
    }

    if let Some(connection_id) = connection_id {
        config.connection_id = connection_id.clone();
        attrs.push(attr("connection_id", connection_id))
    }

    if let Some(port_id) = port_id {
        config.port_id = port_id.clone();
        attrs.push(attr("port_id", port_id))
    }

    if let Some(update_period) = update_period {
        config.update_period = update_period;
        attrs.push(attr("update_period", update_period.to_string()))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn execute_update_votes(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    votes: Vec<ProposalVote>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    VOTERS.save(deps.storage, &votes)?;

    Ok(response(
        "config_update",
        CONTRACT_NAME,
        [attr("total_count", voters.len().to_string())],
    ))
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

    let interchain_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;

    if Some(query_id) == votes_query_id {
        return sudo_proposal_votes(deps, interchain_query_result);
    }

    deps.api.debug(&format!(
        "WASMDEBUG: sudo_kv_query_result query_id: {:?}",
        query_id
    ));

    Ok(Response::default())
}

fn sudo_proposal_votes(
    deps: DepsMut<NeutronQuery>,
    interchain_query_result: QueryRegisteredQueryResultResponse,
) -> ContractResult<Response<NeutronMsg>> {
    let data: GovernmentProposalVotes =
        KVReconstruct::reconstruct(&interchain_query_result.result.kv_results)?;

    deps.api
        .debug(&format!("WASMDEBUG: validator_info_sudo data: {data:?}",));

    Ok(Response::new())
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
    deps.api
        .debug(&format!("WASMDEBUG: proposals_votes_reply call: {msg:?}",));

    let query_id = get_query_id(msg.result)?;

    QUERY_ID.save(deps.storage, &query_id)?;

    Ok(Response::new())
}

fn proposals_votes_remove_reply(deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    deps.api.debug(&format!(
        "WASMDEBUG: proposals_votes_remove_reply call: {msg:?}",
    ));

    QUERY_ID.remove(deps.storage);

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
