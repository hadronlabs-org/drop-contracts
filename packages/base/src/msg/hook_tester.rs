use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use lido_puppeteer_base::msg::ResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetConfig {
        puppeteer_addr: String,
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
    PuppeteerHook(Box<ResponseHookMsg>),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<lido_puppeteer_base::msg::ResponseHookSuccessMsg>)]
    Answers {},
    #[returns(Vec<lido_puppeteer_base::msg::ResponseHookErrorMsg>)]
    Errors {},
}

#[cw_serde]
pub struct MigrateMsg {}
