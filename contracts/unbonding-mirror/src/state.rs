use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub core_contract: String,
    pub source_port: String,
    pub source_channel: String,
    pub ibc_timeout: u64,
    pub prefix: String,
    pub retry_limit: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
