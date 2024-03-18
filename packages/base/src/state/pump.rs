use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use drop_helpers::ica::Ica;

#[cw_serde]
pub struct IBCFees {
    pub recv_fee: Uint128,
    pub ack_fee: Uint128,
    pub timeout_fee: Uint128,
    pub register_fee: Uint128,
}

#[cw_serde]
pub struct PumpTimeout {
    pub local: Option<u64>,
    pub remote: u64,
}

#[cw_serde]
pub struct Config {
    pub dest_address: Option<Addr>,
    pub dest_channel: Option<String>,
    pub dest_port: Option<String>,
    pub connection_id: String,
    pub refundee: Option<Addr>,
    pub owner: Addr,
    pub ibc_fees: IBCFees,
    pub timeout: PumpTimeout,
    pub local_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("core");
pub const ICA: Ica = Ica::new("ica");
pub const ICA_ID: &str = "drop_PUMP";
