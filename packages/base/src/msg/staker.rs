use crate::state::staker::{ConfigOptional, Transaction};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use neutron_sdk::sudo::msg::RequestPacket;

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::pump::Config)]
    Config {},
    #[returns(Uint128)]
    NonStakedBalance {},
    #[returns(Uint128)]
    AllBalance {},
    #[returns(drop_helpers::ica::IcaState)]
    Ica {},
    #[returns(crate::state::staker::TxState)]
    TxState {},
}

#[cw_serde]
pub struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    Stake { items: Vec<(String, Uint128)> },
    IBCTransfer {},
    UpdateConfig { new_config: Box<ConfigOptional> },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub timeout: u64,
    pub remote_denom: String,
    pub base_denom: String,
    pub transfer_channel_id: String,
    pub owner: Option<String>,
    pub allowed_senders: Vec<String>,
    pub min_ibc_transfer: Uint128,
    pub min_staking_amount: Uint128,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ResponseHookMsg {
    Success(ResponseHookSuccessMsg),
    Error(ResponseHookErrorMsg),
}

#[cw_serde]
pub struct ResponseHookSuccessMsg {
    pub request_id: u64,
    pub request: RequestPacket,
    pub transaction: Transaction,
    pub local_height: u64,
    pub remote_height: u64,
}
#[cw_serde]
pub struct ResponseHookErrorMsg {
    pub request_id: u64,
    pub transaction: Transaction,
    pub request: RequestPacket,
    pub details: String,
}

#[cw_serde]
pub enum ReceiverExecuteMsg {
    StakerHook(ResponseHookMsg),
}
