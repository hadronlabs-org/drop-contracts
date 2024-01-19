use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Delegation, Uint128};
use neutron_sdk::sudo::msg::RequestPacket;

use crate::proto::{
    MsgBeginRedelegateResponse, MsgDelegateResponse, MsgExecResponse,
    MsgRedeemTokensforSharesResponse, MsgTokenizeSharesResponse, MsgUndelegateResponse,
};

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
    Delegations {},
}

#[cw_serde]
pub struct TransferReadyBatchMsg {
    pub batch_id: u128,
    pub amount: Uint128,
    pub recipient: String,
}

#[cw_serde]
pub enum ReceiverExecuteMsg {
    PuppeteerHook(ResponseHookMsg),
}

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
    pub answers: Vec<ResponseAnswer>,
}
#[cw_serde]
pub struct ResponseHookErrorMsg {
    pub request_id: u64,
    pub request: RequestPacket,
    pub details: String,
}

#[cw_serde]
pub enum ResponseAnswer {
    DelegateResponse(MsgDelegateResponse),
    UndelegateResponse(MsgUndelegateResponse),
    BeginRedelegateResponse(MsgBeginRedelegateResponse),
    TokenizeSharesResponse(MsgTokenizeSharesResponse),
    RedeemTokensforSharesResponse(MsgRedeemTokensforSharesResponse),
    AuthzExecResponse(MsgExecResponse),
    UnknownResponse {},
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
    ClaimRewardsAndOptionalyTransfer {
        interchain_account_id: String,
        validators: Vec<String>,
        denom: String,
        transfer: Option<TransferReadyBatchMsg>,
    },
}
