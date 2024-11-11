use crate::msg::token::DenomMetadata;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
#[derive(Default)]
pub struct Pause {
    pub mint: bool,
    pub burn: bool,
}

pub const PAUSE: Item<Pause> = Item::new("pause");
pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
pub const DENOM: Item<String> = Item::new("denom");
pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("denom_metadata");
