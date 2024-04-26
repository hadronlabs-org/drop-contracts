use cosmwasm_std::Decimal;
use cw_storage_plus::Map;

pub const PAIRS_PRICES: Map<&(String, String), Decimal> = Map::new("pairs");
