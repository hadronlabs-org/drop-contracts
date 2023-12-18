use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal256;

use crate::state::core::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub voucher_contract: String,
    pub owner: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(Decimal256)]
    ExchangeRate {},
}

#[cw_serde]
pub enum ExecuteMsg {
    Bond {
        receiver: Option<String>,
    },
    Unbond {},
    //permissioned
    UpdateConfig {
        token_contract: Option<String>,
        puppeteer_contract: Option<String>,
        strategy_contract: Option<String>,
        owner: Option<String>,
        ld_denom: Option<String>,
    },
}
#[cw_serde]
pub enum MigrateMsg {}

impl From<InstantiateMsg> for Config {
    fn from(val: InstantiateMsg) -> Self {
        Config {
            token_contract: val.token_contract,
            puppeteer_contract: val.puppeteer_contract,
            strategy_contract: val.strategy_contract,
            voucher_contract: val.voucher_contract,
            owner: val.owner,
            ld_denom: None,
        }
    }
}
