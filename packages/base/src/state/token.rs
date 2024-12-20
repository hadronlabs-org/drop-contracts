use crate::msg::token::DenomMetadata;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const FACTORY_CONTRACT: Item<Addr> = Item::new("factory_contract");
pub const DENOM: Item<String> = Item::new("denom");
pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("denom_metadata");
