use cosmwasm_schema::cw_serde;

use cosmwasm_std::Decimal;
use cw_storage_plus::{Item, Map};
use neutron_sdk::interchain_queries::v045::types::{Proposal, ProposalVote};

#[cw_serde]
pub struct ProposalInfo {
    pub proposal: Proposal,
    pub votes: Option<Vec<ProposalVote>>,
    pub is_spam: bool,
}

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub core_address: String,
    pub proposal_votes_address: String,
    pub validators_set_address: String,
    pub init_proposal: u64,
    pub proposals_prefetch: u64,
    pub veto_spam_threshold: Decimal,
}

#[cw_serde]
pub struct Metrics {
    pub last_proposal: u64,
}

pub const PROPOSALS_REPLY_ID: u64 = 1;

pub const QUERY_ID: Item<u64> = Item::new("query_id");

pub const CONFIG: Item<Config> = Item::new("config");
pub const ACTIVE_PROPOSALS: Item<Vec<u64>> = Item::new("active_proposals");
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
pub const PROPOSALS_VOTES: Map<u64, Vec<ProposalVote>> = Map::new("proposals_votes");
