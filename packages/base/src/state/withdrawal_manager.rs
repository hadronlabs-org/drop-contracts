use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub core_contract: String,
    pub withdrawal_voucher_contract: String,
    pub base_denom: String,
}

pub type Cw721ReceiveMsg = cw721::Cw721ReceiveMsg;

pub const CONFIG: Item<Config> = Item::new("config");
