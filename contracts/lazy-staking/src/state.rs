use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, DenomMetadata};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub factory_addr: String,
    pub base_denom: String,
    pub rewards_receiver: String,
}

pub const CREATE_DENOM_REPLY_ID: u64 = 1;

pub const CONFIG: Item<Config> = Item::new("config");
pub const EXCHANGE_RATE: Item<Decimal> = Item::new("exchange_rate");
pub const REWARDS_RATE: Item<Decimal> = Item::new("rewards_rate");

pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("token_metadata");
pub const DENOM: Item<String> = Item::new("denom");
pub const EXPONENT: Item<u32> = Item::new("exponent");
