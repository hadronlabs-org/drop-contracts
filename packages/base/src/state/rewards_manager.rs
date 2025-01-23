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
pub enum PauseType {
    Switch { exchange_rewards: bool },
    Height { exchange_rewards: u64 },
}

impl Default for PauseType {
    fn default() -> Self {
        PauseType::Switch {
            exchange_rewards: false,
        }
    }
}

#[cw_serde]
#[derive(Default)]
pub struct Pause {
    pub pause: PauseType,
}

pub const PAUSE: Item<Pause> = Item::new("pause");
pub const REWARDS_HANDLERS: Map<String, HandlerConfig> = Map::new("rewards_handlers");
