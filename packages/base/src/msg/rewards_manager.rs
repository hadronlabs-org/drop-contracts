use crate::state::rewards_manager::{HandlerConfig, Pause};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    AddHandler { config: HandlerConfig },
    RemoveHandler { denom: String },
    ExchangeRewards { denoms: Vec<String> },
    SetPause { pause: Pause },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<HandlerConfig>)]
    Handlers {},
    #[returns(Pause)]
    Pause {},
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
}

#[cw_serde]
pub struct MigrateMsg {}
