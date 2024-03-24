use crate::state::provider_proposals::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;
use neutron_sdk::interchain_queries::v045::types::ProposalVote;

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
    #[returns(crate::state::provider_proposals::Config)]
    Config {},
    #[returns(crate::state::provider_proposals::ProposalInfo)]
    GetProposal { proposal_id: u64 },
    #[returns(Vec<crate::state::provider_proposals::ProposalInfo>)]
    GetProposals {},
    #[returns(crate::state::provider_proposals::Metrics)]
    Metrics {},
}

#[cw_serde]
pub struct MigrateMsg {}
