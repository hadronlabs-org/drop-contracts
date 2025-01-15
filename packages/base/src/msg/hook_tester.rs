use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use drop_puppeteer_base::peripheral_hook::ResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetConfig {
        puppeteer_addr: String,
    },
    Undelegate {
        validator: String,
        amount: Uint128,
    },
    Redelegate {
        validator_from: String,
        validator_to: String,
        amount: Uint128,
    },
    TokenizeShare {
        validator: String,
        amount: Uint128,
    },
    RedeemShare {
        validator: String,
        amount: Uint128,
        denom: String,
    },
    PuppeteerHook(Box<ResponseHookMsg>),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<drop_puppeteer_base::peripheral_hook::ResponseHookSuccessMsg>)]
    Answers {},
    #[returns(Vec<drop_puppeteer_base::peripheral_hook::ResponseHookErrorMsg>)]
    Errors {},
}

#[cw_serde]
pub struct MigrateMsg {}
