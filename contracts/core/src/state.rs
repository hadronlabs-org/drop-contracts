use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal256;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub owner: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(Decimal256)]
    ExchangeRate {},
}

pub const CONFIG: Item<Config> = Item::new("config");
