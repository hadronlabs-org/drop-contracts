use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::validatorsstats::{Config, ValidatorState};

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub profile_update_period: u64,
    pub info_update_period: u64,
    pub avg_block_time: u64,
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterStatsQueries { validators: Vec<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(Vec<ValidatorState>)]
    State {},
}

#[cw_serde]
pub struct MigrateMsg {}
