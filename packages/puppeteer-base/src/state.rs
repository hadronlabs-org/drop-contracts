use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Delegation};
use cw_storage_plus::Item;
use neutron_sdk::bindings::msg::IbcFee;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::Transaction;

pub struct PuppeteerBase<'a, T>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
{
    pub config: Item<'a, T>,
    pub state: Item<'a, State>,
    pub recipient_transfers: Item<'a, Vec<Transfer>>,
    pub delegations: Item<'a, (Vec<Delegation>, u64)>,
    pub tx_state: Item<'a, TxState>,
    pub ibc_fee: Item<'a, IbcFee>,
    pub register_fee: Item<'a, Coin>,
}

impl<T> Default for PuppeteerBase<'static, T>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> PuppeteerBase<'a, T>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
{
    pub fn new() -> Self {
        Self {
            config: Item::new("config"),
            state: Item::new("state"),
            recipient_transfers: Item::new("transfers"),
            delegations: Item::new("delegations"),
            tx_state: Item::new("sudo_payload"),
            ibc_fee: Item::new("ibc_fee"),
            register_fee: Item::new("register_fee"),
        }
    }
}

pub trait BaseConfig {
    fn owner(&self) -> &str;
    fn connection_id(&self) -> String;
    fn update_period(&self) -> u64;
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
pub enum IcaState {
    #[default]
    None,
    InProgress,
    Registered,
    Timeout,
}

#[cw_serde]
#[derive(Default)]
pub struct State {
    pub last_processed_height: Option<u64>,
    pub ica: Option<String>,
    pub ica_state: IcaState,
}

#[cw_serde]
#[derive(Default)]
pub enum TxStateStatus {
    #[default]
    Idle,
    InProgress,
    WaitingForAck,
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

pub const SUDO_PAYLOAD_REPLY_ID: u64 = 1;
pub const LOCAL_DENOM: &str = "untrn";
pub const ICA_ID: &str = "LIDO";
