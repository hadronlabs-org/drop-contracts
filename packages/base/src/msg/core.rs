use crate::msg::staker::ResponseHookMsg as StakerResponseHookMsg;
use crate::state::core::{Config, ConfigOptional, NonNativeRewardsItem};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use cw_ownable::cw_ownable_execute;
#[allow(unused_imports)]
use drop_helpers::pause::PauseInfoResponse;
use drop_macros::{pausable, pausable_query};
use drop_puppeteer_base::msg::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub puppeteer_timeout: u64,
    pub strategy_contract: String,
    pub staker_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub validators_set_contract: String,
    pub base_denom: String,
    pub remote_denom: String,
    pub lsm_min_bond_amount: Uint128,
    pub lsm_redeem_threshold: u64,
    pub lsm_redeem_max_interval: u64,  //seconds
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
    pub emergency_address: Option<String>,
    pub min_stake_amount: Uint128,
}

#[cw_serde]
pub struct LastPuppeteerResponse {
    pub response: Option<PuppeteerResponseHookMsg>,
}

#[pausable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(String)]
    Owner {},
    #[returns(cosmwasm_std::Decimal)]
    ExchangeRate {},
    #[returns(crate::state::core::UnbondBatch)]
    UnbondBatch { batch_id: Uint128 },
    #[returns(crate::state::core::ContractState)]
    ContractState {},
    #[returns(LastPuppeteerResponse)]
    LastPuppeteerResponse {},
    #[returns(Vec<NonNativeRewardsItem>)]
    NonNativeRewardsReceivers {},
    #[returns(Vec<(String,(String, Uint128))>)]
    PendingLSMShares {},
    #[returns(Vec<(String,(String, Uint128))>)]
    LSMSharesToRedeem {},
    #[returns(Uint128)]
    TotalBonded {},
}

#[pausable]
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Bond {
        receiver: Option<String>,
        r#ref: Option<String>,
    },
    Unbond {},
    //permissioned
    UpdateConfig {
        new_config: Box<ConfigOptional>,
    },
    UpdateNonNativeRewardsReceivers {
        items: Vec<NonNativeRewardsItem>,
    },
    Tick {},
    PuppeteerHook(Box<PuppeteerResponseHookMsg>),
    StakerHook(Box<StakerResponseHookMsg>),
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
            staker_contract: val.staker_contract,
            withdrawal_voucher_contract: val.withdrawal_voucher_contract,
            withdrawal_manager_contract: val.withdrawal_manager_contract,
            base_denom: val.base_denom,
            remote_denom: val.remote_denom,
            channel: val.channel,
            ld_denom: None,
            idle_min_interval: val.idle_min_interval,
            unbonding_safe_period: val.unbonding_safe_period,
            unbonding_period: val.unbonding_period,
            pump_address: val.pump_address,
            lsm_redeem_threshold: val.lsm_redeem_threshold,
            lsm_redeem_maximum_interval: val.lsm_redeem_max_interval,
            lsm_min_bond_amount: val.lsm_min_bond_amount,
            validators_set_contract: val.validators_set_contract,
            unbond_batch_switch_time: val.unbond_batch_switch_time,
            bond_limit: match val.bond_limit {
                None => None,
                Some(limit) if limit.is_zero() => None,
                Some(limit) => Some(limit),
            },
            fee: val.fee,
            fee_address: val.fee_address,
            emergency_address: val.emergency_address,
            min_stake_amount: val.min_stake_amount,
        }
    }
}
