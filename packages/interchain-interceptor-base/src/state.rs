use cosmwasm_schema::cw_serde;
use cosmwasm_std::Delegation;
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::SudoPayload;

pub struct InterchainIntercaptorBase<'a, T, C>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    C: std::fmt::Debug + Serialize + DeserializeOwned + Clone,
{
    pub config: Item<'a, T>,
    pub state: Item<'a, State>,
    pub recipient_txs: Item<'a, Vec<Transfer>>,
    pub transactions: Item<'a, Vec<C>>,
    pub delegations: Item<'a, (Vec<Delegation>, u64)>,
    pub sudo_payload: Map<'a, (String, u64), SudoPayload<C>>,
    pub reply_id_storage: Item<'a, Vec<u8>>,
    pub ibc_fee: Item<'a, IbcFee>,
}

impl<T, C> Default for InterchainIntercaptorBase<'static, T, C>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    C: std::fmt::Debug + Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T, C> InterchainIntercaptorBase<'a, T, C>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    C: std::fmt::Debug + Serialize + DeserializeOwned + Clone,
{
    pub fn new() -> Self {
        Self {
            config: Item::new("config"),
            state: Item::new("state"),
            recipient_txs: Item::new("txs"),
            transactions: Item::new("transactions"),
            delegations: Item::new("delegations"),
            sudo_payload: Map::new("sudo_payload"),
            reply_id_storage: Item::new("reply_queue_id"),
            ibc_fee: Item::new("ibc_fee"),
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
}

#[cw_serde]
#[derive(Default)]
pub struct State {
    pub last_processed_height: Option<u64>,
    pub ica: Option<String>,
    pub ica_state: IcaState,
}

pub type Recipient = str;

pub const SUDO_PAYLOAD_REPLY_ID: u64 = 1;
pub const LOCAL_DENOM: &str = "untrn";
pub const ICA_ID: &str = "LIDO";
