use cosmwasm_std::DenomMetadata;
use cw_storage_plus::Item;

pub const CREATE_DENOM_REPLY_ID: u64 = 1;

pub const EXPONENT: Item<u32> = Item::new("exponent");
pub const DENOM: Item<String> = Item::new("denom");
pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("denom_metadata");
