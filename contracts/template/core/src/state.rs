use cosmwasm_std::Decimal;
use cw_storage_plus::Item;

pub const EXCHANGE_RATE: Item<Decimal> = Item::new("exchange_rate");