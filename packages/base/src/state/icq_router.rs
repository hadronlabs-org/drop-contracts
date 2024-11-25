use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

use crate::msg::icq_router::{BalancesData, DelegationsData};

#[cw_serde]
pub struct Config {
    pub adapter: Addr,
}

#[cw_serde]
pub struct ConfigOptional {
    pub adapter: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BALANCES: Item<BalancesData> = Item::new("balances");
pub const DELEGATIONS: Item<DelegationsData> = Item::new("delegations");
pub const NON_NATIVE_REWARD_BALANCES: Item<BalancesData> = Item::new("non_native_reward_balances");
