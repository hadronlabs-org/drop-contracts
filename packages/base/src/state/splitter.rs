use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::Item;

#[cw_serde]
#[derive(Default)]
pub struct Config {
    pub receivers: Vec<(String, Uint128)>,
    pub denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
