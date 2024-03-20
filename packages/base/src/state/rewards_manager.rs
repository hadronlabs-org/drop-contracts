use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::Map;

#[cw_serde]
pub struct HandlerConfig {
    pub address: String,
    pub denom: String,
    pub min_rewards: Uint128,
}

pub const REWARDS_HANDLERS: Map<String, HandlerConfig> = Map::new("rewards_handlers");
