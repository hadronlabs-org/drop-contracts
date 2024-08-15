use cw_storage_plus::Item;

use crate::msg::redepmtion_rate_adapter::Config;

pub const CONFIG: Item<Config> = Item::new("config");
