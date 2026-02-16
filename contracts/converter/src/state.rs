use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub from_token: String,
    pub to_token: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const RATE: Item<Decimal> = Item::new("rate");
