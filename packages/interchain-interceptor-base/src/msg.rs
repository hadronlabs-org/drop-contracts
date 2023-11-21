use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Delegation, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    RegisterQuery {},
    RegisterDelegatorDelegationsQuery {
        validators: Vec<String>,
    },
    SetFees {
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
    },
}

#[cw_serde]
pub enum Transaction {
    Delegate {
        interchain_account_id: String,
        validator: String,
        denom: String,
        amount: u128,
    },
    Undelegate {
        interchain_account_id: String,
        validator: String,
        denom: String,
        amount: u128,
    },
    Redelegate {
        interchain_account_id: String,
        validator_from: String,
        validator_to: String,
        denom: String,
        amount: u128,
    },
    WithdrawReward {
        interchain_account_id: String,
        validator: String,
    },
    TokenizeShare {
        interchain_account_id: String,
        validator: String,
        denom: String,
        amount: u128,
    },
    RedeemShare {
        interchain_account_id: String,
        validator: String,
        denom: String,
        amount: u128,
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
