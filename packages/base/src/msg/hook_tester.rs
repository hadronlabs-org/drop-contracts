use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub puppeteer_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
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
