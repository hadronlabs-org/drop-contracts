use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
#[allow(unused_imports)]
use drop_helpers::pause::PauseInfoResponse;
use drop_macros::{pausable, pausable_query};

use crate::state::rewards_manager::HandlerConfig;

#[cw_ownable_execute]
#[pausable]
#[cw_serde]
pub enum ExecuteMsg {
    AddHandler { config: HandlerConfig },
    RemoveHandler { denom: String },
    ExchangeRewards { denoms: Vec<String> },
}

#[cw_ownable_query]
#[pausable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<HandlerConfig>)]
    Handlers {},
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
pub enum MigrateMsg {}
