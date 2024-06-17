use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use drop_puppeteer_base::state::{BalancesAndDelegationsState, BaseConfig};

use crate::msg::puppeteer::{BalancesAndDelegations, MultiBalances};

#[cw_serde]
pub struct ConfigOptional {
    pub connection_id: Option<String>,
    pub port_id: Option<String>,
    pub update_period: Option<u64>,
    pub remote_denom: Option<String>,
    pub allowed_senders: Option<Vec<String>>,
    pub transfer_channel_id: Option<String>,
    pub sdk_version: Option<String>,
    pub timeout: Option<u64>,
}

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64, // update period in seconds for ICQ queries
    pub remote_denom: String,
    pub allowed_senders: Vec<Addr>,
    pub transfer_channel_id: String,
    pub sdk_version: String,
    pub timeout: u64, // timeout for interchain transactions in seconds
}

impl BaseConfig for Config {
    fn connection_id(&self) -> String {
        self.connection_id.clone()
    }

    fn update_period(&self) -> u64 {
        self.update_period
    }
}

#[cw_serde]
pub enum KVQueryType {
    UnbondingDelegations,
    DelegationsAndBalance,
    NonNativeRewardsBalances,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const NON_NATIVE_REWARD_BALANCES: Item<BalancesAndDelegationsState<MultiBalances>> =
    Item::new("non_native_reward_balances");

pub const DELEGATIONS_AND_BALANCE: Item<BalancesAndDelegationsState<BalancesAndDelegations>> =
    Item::new("delegations_and_balance");
