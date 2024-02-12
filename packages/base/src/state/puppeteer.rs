use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use lido_puppeteer_base::state::BaseConfig;
use neutron_sdk::interchain_queries::v045::types::{Balances, Delegations};

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
    Delegations,
    Balance,
}

pub const SUDO_PAYLOAD_REPLY_ID: u64 = 1;
pub const SUDO_IBC_TRANSFER_REPLY_ID: u64 = 2;
pub const SUDO_KV_BALANCE_REPLY_ID: u64 = 3;
pub const SUDO_KV_DELEGATIONS_REPLY_ID: u64 = 4;

pub const DELEGATIONS: Item<(Delegations, u64)> = Item::new("delegations");
pub const BALANCES: Item<(Balances, u64)> = Item::new("balances");
