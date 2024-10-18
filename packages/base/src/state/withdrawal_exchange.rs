use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const WITHDRAWAL_TOKEN_ADDRESS: Item<Addr> = Item::new("withdrawal_token");
