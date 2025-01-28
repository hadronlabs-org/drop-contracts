use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct HandlerConfig {
    pub address: String,
    pub denom: String,
    pub min_rewards: Uint128,
}

#[cw_serde]
#[derive(Default)]
pub struct Pause {
    pub exchange_rewards: u64,
}

pub const PAUSE: Item<Pause> = Item::new("pause");
pub const REWARDS_HANDLERS: Map<String, HandlerConfig> = Map::new("rewards_handlers");
