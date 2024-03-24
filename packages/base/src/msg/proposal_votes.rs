use crate::state::proposal_votes::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub core_address: String,
    pub provider_proposals_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
    UpdateActiveProposals { active_proposals: Vec<u64> },
    UpdateVotersList { voters: Vec<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::proposal_votes::Config)]
    Config {},
    #[returns(crate::state::proposal_votes::Metrics)]
    Metrics {},
}

#[cw_serde]
pub struct MigrateMsg {}
