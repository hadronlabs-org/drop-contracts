use std::collections::HashMap;

use cosmos_sdk_proto::cosmos::gov::v1beta1::ProposalStatus;
use cosmwasm_std::{
    attr, ensure_eq, entry_point, to_json_binary, Attribute, Binary, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Order, Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};

use lido_helpers::answer::response;
use lido_helpers::query_id::get_query_id;
use lido_staking_base::msg::proposal_votes::ExecuteMsg as ProposalVotesExecuteMsg;
use lido_staking_base::msg::provider_proposals::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use lido_staking_base::msg::validatorset::ExecuteMsg as ValidatorSetExecuteMsg;
use lido_staking_base::state::provider_proposals::{
    Config, ConfigOptional, Metrics, ProposalInfo, CONFIG, PROPOSALS, PROPOSALS_REPLY_ID,
    PROPOSALS_VOTES, QUERY_ID,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::{NeutronQuery, QueryRegisteredQueryResultResponse};
use neutron_sdk::interchain_queries::queries::get_raw_interchain_query_result;
use neutron_sdk::interchain_queries::types::KVReconstruct;
use neutron_sdk::interchain_queries::v045::register_queries::{
    new_register_gov_proposal_query_msg, update_register_gov_proposal_query_msg,
};
use neutron_sdk::interchain_queries::v045::types::{GovernmentProposal, Proposal, ProposalVote};
use neutron_sdk::sudo::msg::SudoMsg;

use crate::error::{ContractError, ContractResult};

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let core = deps.api.addr_validate(&msg.core_address)?;
    let validators_set = deps.api.addr_validate(&msg.validators_set_address)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core.as_ref()))?;

    let config = &Config {
        connection_id: msg.connection_id.to_string(),
        port_id: msg.port_id.clone(),
        update_period: msg.update_period,
        core_address: msg.core_address.to_string(),
        proposal_votes_address: None,
        validators_set_address: validators_set.to_string(),
        init_proposal: msg.init_proposal,
        proposals_prefetch: msg.proposals_prefetch,
        veto_spam_threshold: msg.veto_spam_threshold,
    };

    CONFIG.save(deps.storage, config)?;

    let initial_proposals: Vec<u64> =
        (msg.init_proposal..msg.init_proposal + msg.proposals_prefetch).collect();

    let reg_msg = new_register_gov_proposal_query_msg(
        msg.connection_id.to_string(),
        initial_proposals.clone(),
        msg.update_period,
    )?;

    let sub_msg = SubMsg::reply_on_success(reg_msg, PROPOSALS_REPLY_ID);

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("connection_id", msg.connection_id),
            attr("port_id", msg.port_id),
            attr("update_period", msg.update_period.to_string()),
            attr("core_address", msg.core_address),
            attr("validators_set_address", msg.validators_set_address),
            attr("init_proposal", msg.init_proposal.to_string()),
            attr("proposals_prefetch", msg.proposals_prefetch.to_string()),
            attr("veto_spam_threshold", msg.veto_spam_threshold.to_string()),
        ],
    )
    .add_submessage(sub_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::GetProposal { proposal_id } => query_proposal(deps, proposal_id),
        QueryMsg::GetProposals {} => query_proposals(deps),
        QueryMsg::Metrics {} => query_metrics(deps),
    }
}

fn query_config(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

fn query_metrics(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let keys = PROPOSALS.keys(deps.storage, None, None, Order::Ascending);
    let max_key = keys.fold(0u64, |max, current| {
        let current_key = current.unwrap_or_default();
        if current_key > max {
            current_key
        } else {
            max
        }
    });

    to_json_binary(&Metrics {
        last_proposal: max_key,
    })
}

fn query_proposal(deps: Deps<NeutronQuery>, proposal_id: u64) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let proposal: Proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    let votes = PROPOSALS_VOTES
        .may_load(deps.storage, proposal_id)
        .ok()
        .unwrap_or_default();
    to_json_binary(&ProposalInfo {
        proposal: proposal.clone(),
        votes,
        is_spam: is_spam_proposal(&proposal, config.veto_spam_threshold),
    })
}

fn query_proposals(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let proposals: StdResult<Vec<_>> = PROPOSALS
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            item.map(|(_key, value)| {
                let votes = PROPOSALS_VOTES
                    .may_load(deps.storage, value.proposal_id)
                    .ok()
                    .unwrap_or_default();

                ProposalInfo {
                    proposal: value.clone(),
                    votes,
                    is_spam: is_spam_proposal(&value, config.veto_spam_threshold),
                }
            })
        })
        .collect();

    to_json_binary(&proposals?)
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
        ExecuteMsg::UpdateProposalVotes { votes } => execute_update_votes(deps, info, votes),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut msgs: Vec<CosmosMsg<NeutronMsg>> = Vec::new();

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = new_config.core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        config.core_address = core_address.to_string();
        attrs.push(attr("core_address", core_address))
    }

    if let Some(proposal_votes_address) = new_config.proposal_votes_address {
        let proposal_votes_address = deps.api.addr_validate(&proposal_votes_address)?;
        config.proposal_votes_address = Some(proposal_votes_address.to_string());

        let keys = PROPOSALS.keys(deps.storage, None, None, Order::Ascending);
        let proposal_ids = keys
            .map(|key| key.unwrap_or_default())
            .filter(|id| *id != 0)
            .collect::<Vec<u64>>();

        msgs.push(update_voting_proposals_msg(
            proposal_votes_address.to_string(),
            proposal_ids,
        )?);

        attrs.push(attr("proposal_votes_address", proposal_votes_address))
    }

    if let Some(validators_set_address) = new_config.validators_set_address {
        let validators_set_address = deps.api.addr_validate(&validators_set_address)?;
        config.validators_set_address = validators_set_address.to_string();
        attrs.push(attr("validators_set_address", validators_set_address))
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

    if let Some(proposals_prefetch) = new_config.proposals_prefetch {
        config.proposals_prefetch = proposals_prefetch;
        attrs.push(attr("proposals_prefetch", proposals_prefetch.to_string()))
    }

    if let Some(veto_spam_threshold) = new_config.veto_spam_threshold {
        config.veto_spam_threshold = veto_spam_threshold;
        attrs.push(attr("veto_spam_threshold", veto_spam_threshold.to_string()))
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(response("config_update", CONTRACT_NAME, attrs).add_messages(msgs))
}

pub fn execute_update_votes(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    votes: Vec<ProposalVote>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    ensure_eq!(
        config.proposal_votes_address,
        Some(info.sender.to_string()),
        ContractError::Unauthorized {}
    );

    let mut votes_map: HashMap<u64, Vec<ProposalVote>> = HashMap::new();

    for vote in votes.clone() {
        votes_map.entry(vote.proposal_id).or_default().push(vote);
    }

    for (proposal_id, votes) in votes_map.iter() {
        PROPOSALS_VOTES.save(deps.storage, *proposal_id, votes)?;
    }

    Ok(response(
        "config_update",
        CONTRACT_NAME,
        [attr("total_count", votes.len().to_string())],
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

    let proposals_query_id = QUERY_ID.may_load(deps.storage)?;

    let interchain_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;

    if Some(query_id) == proposals_query_id {
        return sudo_proposals_query(deps, interchain_query_result);
    }

    Ok(Response::default())
}

fn sudo_proposals_query(
    deps: DepsMut<NeutronQuery>,
    interchain_query_result: QueryRegisteredQueryResultResponse,
) -> ContractResult<Response<NeutronMsg>> {
    let data: GovernmentProposal =
        KVReconstruct::reconstruct(&interchain_query_result.result.kv_results)?;

    let mut msgs: Vec<CosmosMsg<NeutronMsg>> = Vec::new();
    match data.proposals.first() {
        Some(first_proposal) => {
            if is_proposal_finished(first_proposal) {
                let query_id = QUERY_ID.may_load(deps.storage)?;
                let config = CONFIG.load(deps.storage)?;
                if let Some(query_id) = query_id {
                    let new_proposals: Vec<u64> = (first_proposal.proposal_id
                        ..first_proposal.proposal_id + config.proposals_prefetch)
                        .collect();

                    let reg_msg = CosmosMsg::Custom(update_register_gov_proposal_query_msg(
                        query_id,
                        new_proposals.to_owned(),
                        None,
                        None,
                    )?);

                    msgs.push(reg_msg);

                    let votes = PROPOSALS_VOTES
                        .may_load(deps.storage, first_proposal.proposal_id)
                        .ok()
                        .unwrap_or_default();

                    let update_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: config.validators_set_address,
                        msg: to_json_binary(&ValidatorSetExecuteMsg::UpdateValidatorsVoting {
                            proposal: ProposalInfo {
                                proposal: first_proposal.clone(),
                                votes,
                                is_spam: is_spam_proposal(
                                    first_proposal,
                                    config.veto_spam_threshold,
                                ),
                            },
                        })?,
                        funds: vec![],
                    });

                    msgs.push(update_msg);

                    if let Some(proposal_votes_address) = config.proposal_votes_address {
                        msgs.push(update_voting_proposals_msg(
                            proposal_votes_address,
                            new_proposals,
                        )?);
                    }
                }
            }
        }
        None => deps.api.debug("WASMDEBUG: first_proposal is None"),
    }

    for proposal in data.proposals {
        if proposal.status != ProposalStatus::Unspecified as i32 {
            PROPOSALS.save(deps.storage, proposal.proposal_id, &proposal)?;
        }
    }

    Ok(Response::new().add_messages(msgs))
}

fn is_proposal_finished(proposal: &Proposal) -> bool {
    proposal.status == ProposalStatus::Passed as i32
        || proposal.status == ProposalStatus::Rejected as i32
        || proposal.status == ProposalStatus::Failed as i32
}

fn is_spam_proposal(proposal: &Proposal, veto_spam_threshold: Decimal) -> bool {
    if let Some(final_tally_result) = &proposal.final_tally_result {
        let total_votes = final_tally_result.yes
            + final_tally_result.no
            + final_tally_result.abstain
            + final_tally_result.no_with_veto;

        if total_votes == Uint128::zero() {
            return false;
        }

        return Decimal::from_ratio(final_tally_result.no_with_veto, total_votes)
            > veto_spam_threshold;
    }

    false
}

fn update_voting_proposals_msg(
    proposal_votes_address: String,
    active_proposals: Vec<u64>,
) -> ContractResult<CosmosMsg<NeutronMsg>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proposal_votes_address,
        msg: to_json_binary(&ProposalVotesExecuteMsg::UpdateActiveProposals { active_proposals })?,
        funds: vec![],
    }))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: reply msg: {msg:?}").as_str());

    match msg.id {
        PROPOSALS_REPLY_ID => proposals_votes_reply(deps, env, msg),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn proposals_votes_reply(deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    let query_id = get_query_id(msg.result)?;

    QUERY_ID.save(deps.storage, &query_id)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
