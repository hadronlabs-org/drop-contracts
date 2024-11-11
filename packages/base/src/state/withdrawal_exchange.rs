use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub withdrawal_token_contract: Addr,
    pub withdrawal_voucher_contract: Addr,
    pub denom_prefix: String,
}
pub const CONFIG: Item<Config> = Item::new("config");
