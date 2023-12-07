use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Delegation, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    RegisterQuery {},
    SetFees {
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
        register_fee: Uint128,
    },
}

#[cw_serde]
pub struct SudoPayload<C> {
    pub message: String,
    pub port_id: String,
    pub info: Option<C>,
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
pub struct DelegationsResponse {
    pub delegations: Vec<Delegation>,
    pub last_updated_height: u64,
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    State {},
    Transactions {},
    InterchainTransactions {},
    Delegations {},
}
