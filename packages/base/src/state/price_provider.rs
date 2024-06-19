use cosmwasm_std::Decimal;
use cw_storage_plus::Map;

pub const PRICES: Map<&String, Decimal> = Map::new("pairs");
