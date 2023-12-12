use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}

pub const CONFIG: Item<Config> = Item::new("config");
