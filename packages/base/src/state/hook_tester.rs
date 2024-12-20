use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;
use drop_puppeteer_base::peripheral_hook::{ResponseHookErrorMsg, ResponseHookSuccessMsg};

#[cw_serde]
pub struct Config {
    pub puppeteer_addr: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const ANSWERS: Item<Vec<ResponseHookSuccessMsg>> = Item::new("answers");
pub const ERRORS: Item<Vec<ResponseHookErrorMsg>> = Item::new("errors");
