use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct MsgDelegateResponse {}

#[cw_serde]
pub struct MsgIBCTransfer {}

#[cw_serde]
pub struct MsgSendResponse {}

#[cw_serde]
pub struct MsgUndelegateResponse {
    pub completion_time: Option<Timestamp>,
}

#[cw_serde]
pub struct MsgBeginRedelegateResponse {
    pub completion_time: Option<Timestamp>,
}

#[cw_serde]
pub struct MsgTokenizeSharesResponse {
    pub amount: Option<Coin>,
}
#[cw_serde]
pub struct MsgRedeemTokensforSharesResponse {
    pub amount: Option<Coin>,
}
#[cw_serde]
pub struct MsgExecResponse {
    pub results: Vec<Vec<u8>>,
}

#[cw_serde]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

impl From<prost_types::Timestamp> for Timestamp {
    fn from(ts: prost_types::Timestamp) -> Self {
        Timestamp {
            seconds: ts.seconds,
            nanos: ts.nanos,
        }
    }
}
