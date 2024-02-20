use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::proposal_votes::{Config, Metrics};

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
    UpdateConfig {
        connection_id: Option<String>,
        port_id: Option<String>,
        update_period: Option<u64>,
        core_address: Option<String>,
        provider_proposals_address: Option<String>,
    },
    UpdateActiveProposals {
        active_proposals: Vec<u64>,
    },
    UpdateVotersList {
        voters: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(Metrics)]
    Metrics {},
}

#[cw_serde]
pub struct MigrateMsg {}
