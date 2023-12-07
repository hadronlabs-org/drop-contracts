use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use lido_interchain_interceptor_base::msg::ExecuteMsg as BaseExecuteMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: String,
    pub proxy_address: Addr,
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
        register_fee: Uint128,
    },
    Delegate {
        validator: String,
        amount: Uint128,
        timeout: Option<u64>,
    },
    Undelegate {
        validator: String,
        amount: Uint128,
        timeout: Option<u64>,
    },
    Redelegate {
        validator_from: String,
        validator_to: String,
        amount: Uint128,
        timeout: Option<u64>,
    },
    WithdrawReward {
        validator: String,
        timeout: Option<u64>,
    },
}

impl ExecuteMsg {
    pub fn to_base_enum(&self) -> BaseExecuteMsg {
        match self {
            ExecuteMsg::RegisterICA {} => BaseExecuteMsg::RegisterICA {},
            ExecuteMsg::RegisterQuery {} => BaseExecuteMsg::RegisterQuery {},
            ExecuteMsg::SetFees {
                recv_fee,
                ack_fee,
                timeout_fee,
                register_fee,
            } => BaseExecuteMsg::SetFees {
                recv_fee: *recv_fee,
                ack_fee: *ack_fee,
                timeout_fee: *timeout_fee,
                register_fee: *register_fee,
            },
            _ => unimplemented!(),
        }
    }
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
}

#[cw_serde]
pub struct MigrateMsg {}
