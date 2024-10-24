use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
pub const VALIDATORS_SET_ADDRESS: Item<Addr> = Item::new("validators_set");

// Validator referral code → Validator address
pub const REFS: Map<&str, String> = Map::new("refs");
