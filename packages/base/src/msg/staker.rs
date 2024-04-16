use crate::state::{pump::IBCFees, staker::ConfigOptional};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::pump::Config)]
    Config {},
    #[returns(Uint128)]
    BalanceInProgress {},
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
    Stake {},
    BalanceInProgress {},
    UpdateConfig { new_config: Box<ConfigOptional> },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub dest_address: Option<String>,
    pub dest_channel: Option<String>,
    pub dest_port: Option<String>,
    pub connection_id: String,
    pub ibc_fees: IBCFees,
    pub timeout: u64,
    pub local_denom: String,
    pub remote_denom: String,
    pub owner: Option<String>,
}

#[cw_serde]
pub enum MigrateMsg {}
