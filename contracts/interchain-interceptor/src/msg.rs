use crate::state::{Config, State, Transfer};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    RegisterQuery {},
    SetFees {
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
    },
    Delegate {
        validator: String,
        amount: Uint128,
        timeout: Option<u64>,
    },
}

#[cw_serde]
pub struct DelegateInfo {
    pub interchain_account_id: String,
    pub validator: String,
    pub denom: String,
    pub amount: u128,
}
#[cw_serde]
pub struct SudoPayload {
    pub message: String,
    pub port_id: String,
    pub info: Option<DelegateInfo>,
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

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(State)]
    State {},
    #[returns(Vec<Transfer>)]
    Transactions {},
    #[returns(Vec<DelegateInfo>)]
    Delegations {},
}
