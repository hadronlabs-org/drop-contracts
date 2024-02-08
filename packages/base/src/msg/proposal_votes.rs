use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::proposal_votes::ConfigResponse;

#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub gov_helper_address: String,
}

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub core_address: String,
    pub gov_helper_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        connection_id: Option<String>,
        port_id: String,
        update_period: u64,
        core_address: Option<String>,
        gov_helper_address: Option<String>,
    },
    UpdateActiveProposals {
        active_proposals: Vec<u64>,
    },
    UpdateVotersList {
        voters: Vec<Addr>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
