use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use drop_helpers::pause::PauseInfoResponse;
use drop_macros::{pausable, pausable_query};

use crate::state::rewards_manager::HandlerConfig;

#[pausable]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { owner: Option<String> },
    AddHandler { config: HandlerConfig },
    RemoveHandler { denom: String },
    ExchangeRewards {},
}

#[pausable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Vec<HandlerConfig>)]
    Handlers {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
pub enum MigrateMsg {}
