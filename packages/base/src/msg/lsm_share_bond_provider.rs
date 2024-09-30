use crate::state::lsm_share_bond_provider::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use cosmwasm_std::{Coin, Decimal, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use drop_macros::{bond_provider, bond_provider_query};
use drop_puppeteer_base::msg::ResponseHookMsg as PuppeteerResponseHookMsg;

#[allow(unused_imports)]
use super::core::LastPuppeteerResponse;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub puppeteer_contract: String,
    pub core_contract: String,
    pub validators_set_contract: String,
    pub transfer_channel_id: String,
    pub lsm_redeem_threshold: u64,        //amount of lsm denoms
    pub lsm_redeem_maximum_interval: u64, //seconds
}

#[bond_provider]
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
    PuppeteerHook(Box<PuppeteerResponseHookMsg>),
}

#[bond_provider_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::lsm_share_bond_provider::Config)]
    Config {},
    #[returns(Vec<(String,(String, Uint128))>)]
    PendingLSMShares {},
    #[returns(Vec<(String,(String, Uint128))>)]
    LSMSharesToRedeem {},
    #[returns(LastPuppeteerResponse)]
    LastPuppeteerResponse {},
}

#[cw_serde]
pub struct MigrateMsg {}