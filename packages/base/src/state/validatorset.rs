use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct ConfigOptional {
    pub stats_contract: Option<String>,
    pub provider_proposals_contract: Option<String>,
}

#[cw_serde]
pub struct Config {
    pub stats_contract: Addr,
    pub provider_proposals_contract: Option<Addr>,
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
    pub init_proposal: Option<u64>,
    pub total_passed_proposals: u64,
    pub total_voted_proposals: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const VALIDATORS_SET: Map<String, ValidatorInfo> = Map::new("validators_set");
pub const VALIDATORS_LIST: Item<Vec<ValidatorInfo>> = Item::new("validators_list");
