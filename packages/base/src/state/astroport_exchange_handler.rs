use astroport::router::SwapOperation;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub core_contract: String,
    pub price_provider_contract: String,
    pub cron_address: String,
    pub router_contract: String,
    pub pair_contract: String,
    pub from_denom: String,
    pub min_rewards: Uint128,
    pub max_spread: Decimal,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const SWAP_OPERATIONS: Item<Vec<SwapOperation>> = Item::new("swap_operations");
