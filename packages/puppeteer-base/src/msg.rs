use crate::{
    proto::{
        MsgBeginRedelegateResponse, MsgDelegateResponse, MsgExecResponse, MsgIBCTransfer,
        MsgRedeemTokensforSharesResponse, MsgSendResponse, MsgTokenizeSharesResponse,
        MsgUndelegateResponse,
    },
    state::RedeemShareItem,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Empty, Uint128};
use neutron_sdk::sudo::msg::RequestPacket;
use schemars::JsonSchema;

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
#[derive(QueryResponses)]

pub enum QueryMsg<E = Empty>
where
    E: JsonSchema,
{
    #[returns(crate::state::ConfigResponse)]
    Config {},
    #[returns(drop_helpers::ica::IcaState)]
    Ica {},
    #[returns(Vec<Transaction>)]
    Transactions {},
    #[returns(cosmwasm_std::Binary)]
    Extention { msg: E },
}

#[cw_serde]
pub struct TransferReadyBatchesMsg {
    pub batch_ids: Vec<u128>,
    pub emergency: bool,
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
    pub transaction: Transaction,
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
    IBCTransfer(MsgIBCTransfer),
    TransferResponse(MsgSendResponse),
    UnknownResponse {},
}

#[cw_serde]
pub enum Transaction {
    Delegate {
        interchain_account_id: String,
        denom: String,
        items: Vec<(String, Uint128)>,
    },
    Undelegate {
        interchain_account_id: String,
        items: Vec<(String, Uint128)>,
        denom: String,
        batch_id: u128,
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
    RedeemShares {
        interchain_account_id: String,
        items: Vec<RedeemShareItem>,
    },
    ClaimRewardsAndOptionalyTransfer {
        interchain_account_id: String,
        validators: Vec<String>,
        denom: String,
        transfer: Option<TransferReadyBatchesMsg>,
    },
    IBCTransfer {
        denom: String,
        amount: u128,
        recipient: String,
        reason: IBCTransferReason,
    },
    Transfer {
        interchain_account_id: String,
        items: Vec<(String, cosmwasm_std::Coin)>,
    },
}

#[cw_serde]
pub enum IBCTransferReason {
    LSMShare,
    Stake,
}
