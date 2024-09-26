use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub base_denom: String,
    pub puppeteer_contract: Addr,
    pub core_contract: Addr,
    pub min_ibc_transfer: Uint128,
    pub min_stake_amount: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NON_STAKED_BALANCE: Item<Uint128> = Item::new("current_balance");

pub const PUPPETEER_TRANSFER_REPLY_ID: u64 = 1;
