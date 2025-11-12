use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

use crate::{
    msg::TransferReadyBatchesMsg,
    proto::{
        MsgBeginRedelegateResponse, MsgDelegateResponse, MsgExecResponse, MsgGrantResponse,
        MsgIBCTransfer, MsgRedeemTokensforSharesResponse, MsgSendResponse,
        MsgTokenizeSharesResponse, MsgUndelegateResponse,
    },
    state::RedeemShareItem,
};

#[cw_serde]
pub enum ReceiverExecuteMsg {
    PeripheralHook(ResponseHookMsg),
}

#[cw_serde]
pub enum ResponseHookMsg {
    Success(ResponseHookSuccessMsg),
    Error(ResponseHookErrorMsg),
}

#[cw_serde]
pub struct ResponseHookSuccessMsg {
    pub transaction: Transaction,
    pub local_height: u64,
    pub remote_height: u64,
}

#[cw_serde]
pub struct ResponseHookErrorMsg {
    pub transaction: Transaction,
    pub details: String,
}

#[cw_serde]
pub enum ResponseAnswer {
    GrantDelegateResponse(MsgGrantResponse),
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
        real_amount: u128,
        recipient: String,
        reason: IBCTransferReason,
    },
    Stake {
        amount: Uint128,
    },
    Transfer {
        interchain_account_id: String,
        items: Vec<(String, cosmwasm_std::Coin)>,
    },
    SetupProtocol {
        interchain_account_id: String,
        rewards_withdraw_address: String,
    },
    EnableTokenizeShares {},
    DisableTokenizeShares {},
}

#[cw_serde]
pub enum IBCTransferReason {
    LSMShare,
    Delegate,
}
