use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use lido_puppeteer_base::state::BaseConfig;
use neutron_sdk::interchain_queries::v045::types::Balances;

use crate::msg::puppeteer::BalancesAndDelegations;

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: Addr,
    pub allowed_senders: Vec<Addr>,
    pub proxy_address: Option<Addr>,
    pub transfer_channel_id: String,
}

impl BaseConfig for Config {
    fn owner(&self) -> &str {
        self.owner.as_str()
    }

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

pub const NON_NATIVE_REWARD_BALANCES: Item<(Balances, u64)> =
    Item::new("non_native_reward_balances");

pub const DELEGATIONS_AND_BALANCE: Item<(BalancesAndDelegations, u64)> =
    Item::new("delegations_and_balance");
