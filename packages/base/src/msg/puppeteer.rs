use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use lido_puppeteer_base::{
    msg::{DelegationsResponse, ExecuteMsg as BaseExecuteMsg, TransferReadyBatchMsg},
    state::{State, Transfer},
};

use crate::state::puppeteer::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: String,
    pub allowed_senders: Vec<String>,
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
        reply_to: String,
    },
    Undelegate {
        validator: String,
        amount: Uint128,
        timeout: Option<u64>,
        reply_to: String,
    },
    Redelegate {
        validator_from: String,
        validator_to: String,
        amount: Uint128,
        timeout: Option<u64>,
        reply_to: String,
    },
    TokenizeShare {
        validator: String,
        amount: Uint128,
        timeout: Option<u64>,
        reply_to: String,
    },
    RedeemShare {
        validator: String,
        amount: Uint128,
        denom: String,
        timeout: Option<u64>,
        reply_to: String,
    },
    ClaimRewardsAndOptionalyTransfer {
        validators: Vec<String>,
        transfer: Option<TransferReadyBatchMsg>,
        timeout: Option<u64>,
        reply_to: String,
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
    #[returns(DelegationsResponse)]
    Delegations {},
}
