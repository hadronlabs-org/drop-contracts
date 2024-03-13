use crate::state::core::{Config, ConfigOptional, NonNativeRewardsItem};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use cw_ownable::cw_ownable_execute;
use lido_puppeteer_base::msg::ResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub puppeteer_timeout: u64,
    pub strategy_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub validators_set_contract: String,
    pub base_denom: String,
    pub remote_denom: String,
    pub idle_min_interval: u64,        //seconds
    pub unbonding_period: u64,         //seconds
    pub unbonding_safe_period: u64,    //seconds
    pub unbond_batch_switch_time: u64, //seconds
    pub bond_limit: Option<Uint128>,
    pub pump_address: Option<String>,
    pub channel: String,
    pub owner: String,
    pub fee: Option<Decimal>,
    pub fee_address: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(cosmwasm_std::Decimal)]
    ExchangeRate {},
    #[returns(crate::state::core::UnbondBatch)]
    UnbondBatch { batch_id: Uint128 },
    #[returns(crate::state::core::ContractState)]
    ContractState {},
    #[returns(ResponseHookMsg)]
    LastPuppeteerResponse {},
    #[returns(Vec<NonNativeRewardsItem>)]
    NonNativeRewardsReceivers {},
    #[returns(Uint128)]
    TotalBonded {},
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Bond { receiver: Option<String> },
    Unbond {},
    //permissioned
    UpdateConfig { new_config: Box<ConfigOptional> },
    UpdateNonNativeRewardsReceivers { items: Vec<NonNativeRewardsItem> },
    Tick {},
    PuppeteerHook(Box<ResponseHookMsg>),
    ResetBondedAmount {},
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
            remote_denom: val.remote_denom,
            channel: val.channel,
            owner: val.owner,
            ld_denom: None,
            idle_min_interval: val.idle_min_interval,
            unbonding_safe_period: val.unbonding_safe_period,
            unbonding_period: val.unbonding_period,
            pump_address: val.pump_address,
            validators_set_contract: val.validators_set_contract,
            unbond_batch_switch_time: val.unbond_batch_switch_time,
            bond_limit: val.bond_limit,
            fee: val.fee,
            fee_address: val.fee_address,
        }
    }
}
