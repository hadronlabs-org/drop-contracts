use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, DenomMetadata, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub fee: Decimal,
    pub fee_receiver: Addr,
    pub profit: Decimal,
    pub voucher_contract: Addr,
    pub unbonding_period: u64,
}

pub const BASE_DENOM: &str = "untrn";
pub const CONFIG: Item<Config> = Item::new("config");
pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("token_metadata");
pub const DENOM: Item<String> = Item::new("denom");
pub const CREATE_DENOM_REPLY_ID: u64 = 1;
pub const EXPONENT: Item<u32> = Item::new("exponent");
pub const UNBOND_ID: Item<u64> = Item::new("unbond_id");
