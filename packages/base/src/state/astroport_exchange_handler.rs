use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
pub const CRON_ADDRESS: Item<Addr> = Item::new("cron_address");
pub const FROM_DENOM: Item<String> = Item::new("from_denom");
