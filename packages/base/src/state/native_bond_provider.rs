use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub base_denom: String,
    pub staker_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
