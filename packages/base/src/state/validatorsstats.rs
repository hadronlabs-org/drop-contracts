use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub profile_update_period: u64,
    pub info_update_period: u64,
    pub avg_block_time: u64,
    pub owner: Addr,
}

#[cw_serde]
pub struct KVQueryIds {
    pub signing_info_id: Option<String>,
    pub validator_profile_id: Option<String>,
}

#[cw_serde]
pub struct ValidatorState {
    pub valoper_address: String,
    pub valcons_address: String,
    pub last_processed_remote_height: Option<u64>,
    pub last_processed_local_height: Option<u64>,
    pub last_validated_height: Option<u64>,
    pub last_commission_in_range: Option<u64>,
    pub uptime: Decimal,
    pub tombstone: bool,
    pub prev_jailed_state: bool,
    pub jailed_number: Option<u64>,
}

#[cw_serde]
pub struct State {
    pub validators: Vec<ValidatorState>,
}

#[cw_serde]
pub struct ValidatorMissedBlocksForPeriod {
    pub address: String,
    pub missed_blocks: u64,
}

#[cw_serde]
pub struct MissedBlocks {
    pub remote_height: u64,
    pub timestamp: u64,
    pub validators: Vec<ValidatorMissedBlocksForPeriod>,
}

pub const VALIDATOR_PROFILE_REPLY_ID: u64 = 1;
pub const SIGNING_INFO_REPLY_ID: u64 = 2;

pub const CONFIG: Item<Config> = Item::new("config");
pub const MISSED_BLOCKS: Item<Vec<MissedBlocks>> = Item::new("missed_blocks");
pub const STATE_MAP: Map<String, ValidatorState> = Map::new("state_map");
pub const VALIDATOR_PROFILE_QUERY_ID: Item<u64> = Item::new("validator_profile_query_id");
pub const SIGNING_INFO_QUERY_ID: Item<u64> = Item::new("signin_info_query_id");
pub const VALCONS_TO_VALOPER: Map<String, String> = Map::new("valcons_to_valoper");
