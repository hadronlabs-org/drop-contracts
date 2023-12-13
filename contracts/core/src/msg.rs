use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use lido_staking_base::msg::CoreInstantiateMsg;

use crate::state::Config;

impl From<CoreInstantiateMsg> for Config {
    fn from(val: CoreInstantiateMsg) -> Self {
        Config {
            token_contract: val.token_contract,
            puppeteer_contract: val.puppeteer_contract,
            strategy_contract: val.strategy_contract,
            owner: val.owner,
        }
    }
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
