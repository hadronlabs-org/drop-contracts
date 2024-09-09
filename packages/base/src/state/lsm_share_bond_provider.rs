use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub puppeteer_contract: Addr,
    pub core_contract: Addr,
    pub validators_set_contract: Addr,
    pub transfer_channel_id: String,
    pub lsm_redeem_threshold: u64,        //amount of lsm denoms
    pub lsm_redeem_maximum_interval: u64, //seconds
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL_LSM_SHARES: Item<u128> = Item::new("total_lsm_shares_v0");
/// (local_denom, (remote_denom, shares_amount, real_amount))
pub const PENDING_LSM_SHARES: Map<String, (String, Uint128, Uint128)> =
    Map::new("pending_lsm_shares_v0");
/// (local_denom, (remote_denom, shares_amount, real_amount))
pub const LSM_SHARES_TO_REDEEM: Map<String, (String, Uint128, Uint128)> =
    Map::new("lsm_shares_to_redeem_v0");
pub const LAST_LSM_REDEEM: Item<u64> = Item::new("last_lsm_redeem_v0");
