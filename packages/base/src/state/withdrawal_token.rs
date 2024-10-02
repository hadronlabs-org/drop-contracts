use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
pub const WITHDRAWAL_MANAGER_ADDRESS: Item<Addr> = Item::new("withdrawal_manager");
pub const DENOM_PREFIX: Item<String> = Item::new("denom_prefix");
