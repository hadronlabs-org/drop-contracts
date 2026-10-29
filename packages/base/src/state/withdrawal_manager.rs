use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub core_contract: Addr,
    pub withdrawal_voucher_contract: Addr,
    pub base_denom: String,
}

pub type Cw721ReceiveMsg = cw721::Cw721ReceiveMsg;

pub const CONFIG: Item<Config> = Item::new("config");
/// Share of the voucher entitlement paid out on withdrawal (haircut). Absent means 1.
pub const PAYOUT_RATIO: Item<Decimal> = Item::new("payout_ratio");
