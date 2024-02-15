use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use lido_puppeteer_base::msg::{ExecuteMsg as BaseExecuteMsg, TransferReadyBatchMsg};
use neutron_sdk::interchain_queries::v045::types::{Balances, Delegations};

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: String,
    pub allowed_senders: Vec<String>,
    pub transfer_channel_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    RegisterQuery {},
    RegisterDelegatorDelegationsQuery {
        validators: Vec<String>,
    },
    RegisterDelegatorUnbondingDelegationsQuery {
        validators: Vec<String>,
    },
    RegisterBalanceQuery {},
    SetFees {
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
        register_fee: Uint128,
    },
    Delegate {
        items: Vec<(String, Uint128)>,
        timeout: Option<u64>,
        reply_to: String,
    },
    Undelegate {
        items: Vec<(String, Uint128)>,
        batch_id: u128,
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
    IBCTransfer {
        timeout: u64,
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

pub type DelegationsResponse = (Delegations, u64);

pub type BalancesResponse = (Balances, u64);

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryExtMsg {
    #[returns(DelegationsResponse)]
    Delegations {},
    #[returns(BalancesResponse)]
    Balances {},
    #[returns(Vec<lido_puppeteer_base::state::UnbondingDelegation>)]
    UnbondingDelegations {},
}
