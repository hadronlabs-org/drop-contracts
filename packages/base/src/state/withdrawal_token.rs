use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
pub const WITHDRAWAL_MANAGER_ADDRESS: Item<Addr> = Item::new("withdrawal_manager");
pub const WITHDRAWAL_EXCHANGE_ADDRESS: Item<Addr> = Item::new("withdrawal_exchange");
pub const IS_INIT_STATE: Item<bool> = Item::new("init_state");
pub const NEXT_BATCH_TO_PREMINT: Item<Option<Uint128>> = Item::new("next_batch_to_premint");
pub const DENOM_PREFIX: Item<String> = Item::new("denom_prefix");
