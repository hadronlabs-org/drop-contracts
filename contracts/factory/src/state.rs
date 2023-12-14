use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub token_code_id: u64,
    pub core_code_id: u64,
    pub owner: String,
    pub salt: String,
    pub subdenom: String,
}

#[cw_serde]
pub struct State {
    pub token_contract: String,
    pub core_contract: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");