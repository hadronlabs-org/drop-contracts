use cosmwasm_schema::cw_serde;

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct ConfigResponse {
    pub core_address: String,
    pub gov_helper_address: String,
}

pub const CORE_ADDRESS: Item<Addr> = Item::new("core_address");
pub const GOV_HELPER_ADDRESS: Item<Addr> = Item::new("gov_helper_address");
pub const ACTIVE_PROPOSALS: Item<Vec<u64>> = Item::new("active_proposals");
pub const VOTERS: Item<Vec<Addr>> = Item::new("voters");
