use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;

use crate::msg::{DelegateInfo, SudoPayload};

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
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
pub struct State {
    pub last_processed_height: Option<u64>,
    pub ica: Option<String>,
}

pub type Recipient = str;

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const RECIPIENT_TXS: Item<Vec<Transfer>> = Item::new("txs");
pub const IBC_FEE: Item<IbcFee> = Item::new("ibc_fee");
pub const REPLY_ID_STORAGE: Item<Vec<u8>> = Item::new("reply_queue_id");
pub const SUDO_PAYLOAD: Map<(String, u64), SudoPayload> = Map::new("sudo_payload");
pub const DELEGATIONS: Item<Vec<DelegateInfo>> = Item::new("delegations");
