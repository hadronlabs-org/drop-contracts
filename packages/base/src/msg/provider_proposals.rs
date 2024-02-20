use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;
use neutron_sdk::interchain_queries::v045::types::ProposalVote;

use crate::state::provider_proposals::{Config, ConfigOptional, Metrics, ProposalInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub core_address: String,
    pub validators_set_address: String,
    pub init_proposal: u64,
    pub proposals_prefetch: u64,
    pub veto_spam_threshold: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
    UpdateProposalVotes { votes: Vec<ProposalVote> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(ProposalInfo)]
    GetProposal { proposal_id: u64 },
    #[returns(Vec<ProposalInfo>)]
    GetProposals {},
    #[returns(Metrics)]
    Metrics {},
}

#[cw_serde]
pub struct MigrateMsg {}
