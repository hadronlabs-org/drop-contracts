use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
