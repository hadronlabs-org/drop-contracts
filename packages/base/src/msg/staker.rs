use crate::state::staker::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use drop_helpers::interchain::IBCFees;

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::pump::Config)]
    Config {},
    #[returns(Uint128)]
    NonStakedBalance {},
    #[returns(drop_helpers::ica::IcaState)]
    Ica {},
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
    pub ibc_fees: IBCFees,
    pub timeout: u64,
    pub remote_denom: String,
    pub base_denom: String,
    pub transfer_channel_id: String,
    pub owner: Option<String>,
    pub allowed_senders: Vec<String>,
}

#[cw_serde]
pub enum MigrateMsg {}
