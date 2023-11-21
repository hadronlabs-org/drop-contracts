use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

use lido_interchain_interceptor_base::msg::ExecuteMsg as BaseExecuteMsg;

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
    TokenizeShare {
        validator: String,
        amount: Uint128,
        timeout: Option<u64>,
    },
    RedeemShare {
        validator: String,
        amount: Uint128,
        denom: String,
        timeout: Option<u64>,
    },
}

impl ExecuteMsg {
    pub fn to_base_enum(&self) -> BaseExecuteMsg {
        match self {
            ExecuteMsg::RegisterICA {} => BaseExecuteMsg::RegisterICA {},
            ExecuteMsg::RegisterQuery {} => BaseExecuteMsg::RegisterQuery {},
            ExecuteMsg::RegisterDelegatorDelegationsQuery { validators } => {
                BaseExecuteMsg::RegisterDelegatorDelegationsQuery {
                    validators: validators.to_vec(),
                }
            }
            ExecuteMsg::SetFees {
                recv_fee,
                ack_fee,
                timeout_fee,
            } => BaseExecuteMsg::SetFees {
                recv_fee: *recv_fee,
                ack_fee: *ack_fee,
                timeout_fee: *timeout_fee,
            },
            _ => panic!("Not implemented"),
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
pub struct MigrateMsg {}
