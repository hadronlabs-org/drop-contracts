use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub stats_contract: Addr,
}

#[cw_serde]
pub struct ValidatorInfo {
    pub valoper_address: String,
    pub weight: u64,
    pub last_processed_remote_height: Option<u64>,
    pub last_processed_local_height: Option<u64>,
    pub last_validated_height: Option<u64>,
    pub last_commission_in_range: Option<u64>,
    pub uptime: Decimal,
    pub tombstone: bool,
    pub jailed_number: Option<u64>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const VALIDATORS_SET: Map<String, ValidatorInfo> = Map::new("validators_set");
