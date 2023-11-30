use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub profile_update_period: u64,
    pub info_update_period: u64,
    pub owner: Addr,
}

#[cw_serde]
#[derive(Default)]
pub struct State {
    pub last_processed_height: Option<u64>,
    pub last_validated_height: Option<u64>,
    pub last_active: Option<u64>,
    pub last_commission_in_range: Option<u64>,
    pub uptime: u64,
    pub tombstone: bool,
    pub jailed_number: Option<u64>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(State)]
    State {},
}

pub const VALIDATOR_PROFILE_REPLY_ID: u64 = 1;
pub const SIGNING_INFO_REPLY_ID: u64 = 2;

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const VALIDATOR_PROFILE_QUERY_ID: Item<Option<u64>> = Item::new("validator_profile_query_id");
pub const SIGNING_INFO_QUERY_ID: Item<Option<u64>> = Item::new("signin_info_query_id");
