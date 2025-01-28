use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Map;

#[cw_serde]
#[derive(Default)]
pub struct User {
    pub rewards_address: Option<Addr>,
    pub rewards: Vec<Coin>,
}

pub const USERS: Map<&Addr, User> = Map::new("users");
