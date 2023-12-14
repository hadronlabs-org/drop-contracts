use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

use crate::state::core::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bond {},
    Unbond {
        amount: Uint128,
    },
    //permissioned
    UpdateConfig {
        token_contract: Option<String>,
        puppeteer_contract: Option<String>,
        strategy_contract: Option<String>,
        owner: Option<String>,
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
            owner: val.owner,
        }
    }
}
