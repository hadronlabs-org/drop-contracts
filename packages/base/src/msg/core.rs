use crate::{
    error::core::ContractResult,
    msg::staker::ResponseHookMsg as StakerResponseHookMsg,
    state::core::{Config, ConfigOptional},
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Deps, Uint128, Uint64};
use cw_ownable::cw_ownable_execute;
#[allow(unused_imports)]
use drop_helpers::pause::PauseInfoResponse;
use drop_macros::{pausable, pausable_query};
use drop_puppeteer_base::msg::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub staker_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub validators_set_contract: String,
    pub base_denom: String,
    pub remote_denom: String,
    pub lsm_min_bond_amount: Uint128,
    pub lsm_redeem_threshold: u64,     //amount of lsm denoms
    pub lsm_redeem_max_interval: u64,  //seconds
    pub idle_min_interval: u64,        //seconds
    pub unbonding_period: u64,         //seconds
    pub unbonding_safe_period: u64,    //seconds
    pub unbond_batch_switch_time: u64, //seconds
    pub pump_ica_address: Option<String>,
    pub transfer_channel_id: String,
    pub owner: String,
    pub emergency_address: Option<String>,
    pub min_stake_amount: Uint128,
    pub icq_update_delay: u64, // blocks
}

impl InstantiateMsg {
    pub fn into_config(self, deps: Deps) -> ContractResult<Config> {
        Ok(Config {
            token_contract: deps.api.addr_validate(&self.token_contract)?,
            puppeteer_contract: deps.api.addr_validate(&self.puppeteer_contract)?,
            strategy_contract: deps.api.addr_validate(&self.strategy_contract)?,
            staker_contract: deps.api.addr_validate(&self.staker_contract)?,
            withdrawal_voucher_contract: deps
                .api
                .addr_validate(&self.withdrawal_voucher_contract)?,
            withdrawal_manager_contract: deps
                .api
                .addr_validate(&self.withdrawal_manager_contract)?,
            base_denom: self.base_denom,
            remote_denom: self.remote_denom,
            idle_min_interval: self.idle_min_interval,
            unbonding_safe_period: self.unbonding_safe_period,
            unbonding_period: self.unbonding_period,
            pump_ica_address: self.pump_ica_address,
            transfer_channel_id: self.transfer_channel_id,
            lsm_redeem_threshold: self.lsm_redeem_threshold,
            lsm_redeem_maximum_interval: self.lsm_redeem_max_interval,
            lsm_min_bond_amount: self.lsm_min_bond_amount,
            validators_set_contract: deps.api.addr_validate(&self.validators_set_contract)?,
            unbond_batch_switch_time: self.unbond_batch_switch_time,
            emergency_address: self.emergency_address,
            min_stake_amount: self.min_stake_amount,
            icq_update_delay: self.icq_update_delay,
        })
    }
}

#[cw_serde]
pub struct LastPuppeteerResponse {
    pub response: Option<PuppeteerResponseHookMsg>,
}
#[cw_serde]
pub struct LastStakerResponse {
    pub response: Option<StakerResponseHookMsg>,
}

#[cw_serde]
pub struct FailedBatchResponse {
    pub response: Option<u128>,
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
    #[returns(Uint128)]
    CurrentUnbondBatch {},
    #[returns(crate::state::core::UnbondBatch)]
    UnbondBatch { batch_id: Uint128 },
    #[returns(crate::state::core::UnbondBatchesResponse)]
    UnbondBatches {
        limit: Option<Uint64>,
        page_key: Option<Uint128>,
    },
    #[returns(crate::state::core::ContractState)]
    ContractState {},
    #[returns(LastPuppeteerResponse)]
    LastPuppeteerResponse {},
    #[returns(LastStakerResponse)]
    LastStakerResponse {},
    #[returns(Vec<(String,(String, Uint128))>)]
    PendingLSMShares {},
    #[returns(Vec<(String,(String, Uint128))>)]
    LSMSharesToRedeem {},
    #[returns(Uint128)]
    TotalBonded {},
    #[returns(Uint128)]
    TotalLSMShares {},
    #[returns(FailedBatchResponse)]
    FailedBatch {},
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
    UpdateWithdrawnAmount {
        batch_id: u128,
        withdrawn_amount: Uint128,
    },
    Tick {},
    PuppeteerHook(Box<PuppeteerResponseHookMsg>),
    StakerHook(Box<StakerResponseHookMsg>),
    ProcessEmergencyBatch {
        batch_id: u128,
        unbonded_amount: Uint128,
    },
}

#[cw_serde]
pub struct MigrateMsg {}
