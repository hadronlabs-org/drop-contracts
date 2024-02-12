use cosmwasm_schema::{cw_serde, QueryResponses};
use neutron_sdk::interchain_queries::v045::types::ProposalVote;

use crate::state::proposal_votes::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub core_address: String,
    pub proposal_votes_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        connection_id: Option<String>,
        port_id: Option<String>,
        update_period: Option<u64>,
        core_address: Option<String>,
        proposal_votes_address: Option<String>,
    },
    UpdateProposalVotes {
        votes: Vec<ProposalVote>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
