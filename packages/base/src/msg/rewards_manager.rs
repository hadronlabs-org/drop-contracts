use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::rewards_manager::HandlerConfig;

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { core_address: Option<String> },
    AddHandler { config: HandlerConfig },
    RemoveHandler { denom: String },
    ExchangeRewards {},
}

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
    pub core_address: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
}

#[cw_serde]
pub enum MigrateMsg {}
