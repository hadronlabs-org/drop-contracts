use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub remote_denom: String,
    pub base_denom: String,
    pub puppeteer_contract: String,
    pub core_contract: String,
    pub min_ibc_transfer: Uint128,
}

#[cw_serde]
pub struct ConfigOptional {
    pub puppeteer_contract: Option<String>,
    pub core_contract: Option<String>,
    pub min_ibc_transfer: Option<Uint128>,
}

pub const CONFIG: Item<Config> = Item::new("core");
pub const NON_STAKED_BALANCE: Item<Uint128> = Item::new("current_balance");
pub const PUPPETEER_TRANSFER_REPLY_ID: u64 = 1;
