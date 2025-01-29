use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub base_denom: String,
    pub puppeteer_contract: Addr,
    pub core_contract: Addr,
    pub strategy_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
