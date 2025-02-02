use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub core_contract: String,
    pub source_port: String,
    pub source_channel: String,
    pub ibc_timeout: u64,
    pub prefix: String,
}

#[cw_serde]
pub struct ConfigOptional {
    pub core_contract: Option<String>,
    pub source_port: Option<String>,
    pub source_channel: Option<String>,
    pub ibc_timeout: Option<u64>,
    pub prefix: Option<String>,
}

#[cw_serde]
pub struct TimeoutRange {
    pub from: u64,
    pub to: u64,
}

pub const BOND_REPLY_ID: u64 = 1;

pub const REPLY_RECEIVER: Item<String> = Item::new("reply_receiver");
pub const CONFIG: Item<Config> = Item::new("config");
pub const FAILED_TRANSFERS: Map<String, Vec<Coin>> = Map::new("failed_transfers");
pub const TIMEOUT_RANGE: TimeoutRange = TimeoutRange {
    from: 0,
    to: 2592000, // 30d
};
