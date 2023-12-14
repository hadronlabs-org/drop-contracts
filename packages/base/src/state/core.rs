use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub owner: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
