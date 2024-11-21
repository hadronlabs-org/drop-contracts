use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DenomMetadata};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub unbonding_period: u64,
    pub withdrawal_voucher: Addr,
}

pub const CREATE_DENOM_REPLY_ID: u64 = 1;
pub const BASE_DENOM: &str = "untrn";
pub const SALT: &str = "salt";

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("token_metadata");
pub const DENOM: Item<String> = Item::new("denom");
pub const EXPONENT: Item<u32> = Item::new("exponent");
pub const UNBOND_ID: Item<u64> = Item::new("unbond_id");
