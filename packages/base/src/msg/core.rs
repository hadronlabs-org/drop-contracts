use crate::state::core::{Config, UnbondBatch};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub base_denom: String,
    pub owner: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(Decimal)]
    ExchangeRate {},
    #[returns(UnbondBatch)]
    UnbondBatch { batch_id: Uint128 },
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
    FakeProcessBatch {
        batch_id: Uint128,
        unbonded_amount: Uint128,
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
            withdrawal_voucher_contract: val.withdrawal_voucher_contract,
            withdrawal_manager_contract: val.withdrawal_manager_contract,
            base_denom: val.base_denom,
            owner: val.owner,
            ld_denom: None,
        }
    }
}
