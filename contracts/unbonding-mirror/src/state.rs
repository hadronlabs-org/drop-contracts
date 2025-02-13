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

#[cw_serde]
pub struct ConfigOptional {
    pub core_contract: Option<String>,
    pub source_port: Option<String>,
    pub source_channel: Option<String>,
    pub ibc_timeout: Option<u64>,
    pub prefix: Option<String>,
    pub retry_limit: Option<u64>,
}

#[cw_serde]
pub struct TimeoutRange {
    pub from: u64,
    pub to: u64,
}

const TIMEOUT_30D: u64 = 2592000;
pub const CONFIG: Item<Config> = Item::new("config");
pub const TIMEOUT_RANGE: TimeoutRange = TimeoutRange {
    from: 0,
    to: TIMEOUT_30D,
};
