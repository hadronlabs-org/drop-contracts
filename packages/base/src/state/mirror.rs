use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
#[derive(Default)]
pub enum ReturnType {
    #[default]
    Remote,
    Local,
}

impl std::fmt::Display for ReturnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReturnType::Remote => write!(f, "Remote"),
            ReturnType::Local => write!(f, "Local"),
        }
    }
}

#[cw_serde]
#[derive(Default)]
pub enum BondState {
    #[default]
    Initiated,
    Bonded,
    Sent,
}

impl std::fmt::Display for BondState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BondState::Initiated => write!(f, "Initiated"),
            BondState::Bonded => write!(f, "Bonded"),
            BondState::Sent => write!(f, "Sent"),
        }
    }
}

#[cw_serde]
pub struct BondItem {
    pub receiver: String,
    pub backup: Option<String>,
    pub amount: Uint128,
    pub received: Option<Coin>,
    pub return_type: ReturnType,
    pub state: BondState,
}

#[cw_serde]
pub struct Config {
    pub core_contract: String,
    pub source_port: String,
    pub source_channel: String,
    pub ibc_timeout: u64,
    pub prefix: String,
}
#[cw_serde]
pub struct ConfigOptional {
    pub core_contract: Option<String>,
    pub source_port: Option<String>,
    pub source_channel: Option<String>,
    pub ibc_timeout: Option<u64>,
    pub prefix: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BONDS: Map<u64, BondItem> = Map::new("bonds");
pub const COUNTER: Item<u64> = Item::new("counter");
