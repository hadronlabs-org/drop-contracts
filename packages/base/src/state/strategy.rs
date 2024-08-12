use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const PUPPETEER_ADDRESS: Item<Addr> = Item::new("puppeteer");
pub const VALIDATOR_SET_ADDRESS: Item<Addr> = Item::new("validator_set");
pub const DISTRIBUTION_ADDRESS: Item<Addr> = Item::new("distribution");
pub const DENOM: Item<String> = Item::new("denom");
