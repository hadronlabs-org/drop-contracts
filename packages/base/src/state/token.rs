use crate::msg::token::DenomMetadata;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
pub const DENOM: Item<String> = Item::new("denom");
pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("denom_metadata");
