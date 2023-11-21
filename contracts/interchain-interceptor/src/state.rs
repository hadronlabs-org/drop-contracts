use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use lido_interchain_interceptor_base::{
    msg::{DelegationsResponse, Transaction},
    state::{HasOwner, State, Transfer},
};
use neutron_sdk::bindings::msg::IbcFee;

use crate::msg::SudoPayload;

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: Addr,
}

impl HasOwner for Config {
    fn owner(&self) -> &str {
        self.owner.as_str()
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(State)]
    State {},
    #[returns(Vec<Transfer>)]
    Transactions {},
    #[returns(Vec<Transaction>)]
    InterchainTransactions {},
    #[returns(DelegationsResponse)]
    Delegations {},
}

pub type Recipient = str;

pub const IBC_FEE: Item<IbcFee> = Item::new("ibc_fee");
pub const REPLY_ID_STORAGE: Item<Vec<u8>> = Item::new("reply_queue_id");
pub const SUDO_PAYLOAD: Map<(String, u64), SudoPayload> = Map::new("sudo_payload");
