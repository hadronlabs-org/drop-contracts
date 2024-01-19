use crate::state::core::{Config, UnbondBatch};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use lido_puppeteer_base::msg::ResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub puppeteer_timeout: u64,
    pub strategy_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub validator_set_contract: String,
    pub base_denom: String,
    pub idle_min_interval: u64,     //seconds
    pub unbonding_period: u64,      //seconds
    pub unbonding_safe_period: u64, //seconds
    pub pump_address: Option<String>,
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
        tick_min_interval: Option<u64>,
    },
    FakeProcessBatch {
        batch_id: Uint128,
        unbonded_amount: Uint128,
    },
    Tick {},
    PuppeteerHook(Box<ResponseHookMsg>),
}

#[cw_serde]
pub enum MigrateMsg {}

impl From<InstantiateMsg> for Config {
    fn from(val: InstantiateMsg) -> Self {
        Config {
            token_contract: val.token_contract,
            puppeteer_contract: val.puppeteer_contract,
            puppeteer_timeout: val.puppeteer_timeout,
            strategy_contract: val.strategy_contract,
            withdrawal_voucher_contract: val.withdrawal_voucher_contract,
            withdrawal_manager_contract: val.withdrawal_manager_contract,
            base_denom: val.base_denom,
            owner: val.owner,
            ld_denom: None,
            idle_min_interval: val.idle_min_interval,
            unbonding_safe_period: val.unbonding_safe_period,
            unbonding_period: val.unbonding_period,
            pump_address: val.pump_address,
            validator_set_contract: val.validator_set_contract,
        }
    }
}
