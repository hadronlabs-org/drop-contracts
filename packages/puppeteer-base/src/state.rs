use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_storage_plus::{Item, Map};
use lido_helpers::ica::Ica;
use neutron_sdk::bindings::msg::IbcFee;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::Transaction;

pub struct PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub config: Item<'a, T>,
    pub ica: Ica<'a>,
    pub recipient_transfers: Item<'a, Vec<Transfer>>,
    pub transfer_channel_id: Item<'a, String>,
    pub tx_state: Item<'a, TxState>,
    pub ibc_fee: Item<'a, IbcFee>,
    pub register_fee: Item<'a, Coin>,
    pub kv_queries: Map<'a, u64, U>,
}

impl<T, U> Default for PuppeteerBase<'static, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T, U> PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub fn new() -> Self {
        Self {
            config: Item::new("config"),
            ica: Ica::new("ica"),
            recipient_transfers: Item::new("transfers"),
            tx_state: Item::new("sudo_payload"),
            ibc_fee: Item::new("ibc_fee"),
            register_fee: Item::new("register_fee"),
            transfer_channel_id: Item::new("transfer_channel_id"),
            kv_queries: Map::new("kv_queries"),
        }
    }
}

pub trait BaseConfig {
    fn owner(&self) -> &str;
    fn connection_id(&self) -> String;
    fn update_period(&self) -> u64;
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub connection_id: String,
    pub update_period: u64,
}

#[cw_serde]
pub struct Transfer {
    pub recipient: String,
    pub sender: String,
    pub denom: String,
    pub amount: String,
}

#[cw_serde]
#[derive(Default)]
pub enum TxStateStatus {
    #[default]
    Idle,
    InProgress,
    WaitingForAck,
}

impl std::fmt::Display for TxStateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxStateStatus::Idle => write!(f, "Idle"),
            TxStateStatus::InProgress => write!(f, "InProgress"),
            TxStateStatus::WaitingForAck => write!(f, "WaitingForAck"),
        }
    }
}

#[cw_serde]
#[derive(Default)]
pub struct TxState {
    pub status: TxStateStatus,
    pub seq_id: Option<u64>,
    pub transaction: Option<Transaction>,
    pub reply_to: Option<String>,
}

pub type Recipient = str;
pub const LOCAL_DENOM: &str = "untrn";
pub const ICA_ID: &str = "LIDO";
