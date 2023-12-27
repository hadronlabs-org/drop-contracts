use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub puppeteer_addr: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
