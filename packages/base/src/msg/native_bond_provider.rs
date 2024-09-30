use crate::state::native_bond_provider::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use cosmwasm_std::{Coin, Decimal, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use drop_macros::{bond_provider, bond_provider_query};
use drop_puppeteer_base::msg::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_denom: String,
    pub min_ibc_transfer: Uint128,
    pub min_stake_amount: Uint128,
    pub puppeteer_contract: String,
    pub core_contract: String,
    pub strategy_contract: String,
}

#[bond_provider]
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
    PuppeteerTransfer {},
    PuppeteerHook(Box<PuppeteerResponseHookMsg>),
}

#[bond_provider_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::native_bond_provider::Config)]
    Config {},
    #[returns(Uint128)]
    NonStakedBalance {},
    #[returns(Uint128)]
    AllBalance {},
    #[returns(crate::state::native_bond_provider::TxState)]
    TxState {},
}

#[cw_serde]
pub struct MigrateMsg {}
