use cw_storage_plus::{Item, Map};

pub const ACCOUNTS: Map<String, String> = Map::new("accounts");
pub const PREFIX: Item<String> = Item::new("prefix");
