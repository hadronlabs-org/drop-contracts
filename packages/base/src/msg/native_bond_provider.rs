use crate::state::native_bond_provider::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use cosmwasm_std::{Coin, Decimal, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use drop_macros::{bond_provider, bond_provider_query};
use drop_puppeteer_base::peripheral_hook::ResponseHookMsg as PuppeteerResponseHookMsg;

#[allow(unused_imports)]
use super::core::LastPuppeteerResponse;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_denom: String,
    pub min_ibc_transfer: Uint128,
    pub min_stake_amount: Uint128,
    pub factory_contract: String,
    pub port_id: String,
    pub transfer_channel_id: String,
    pub timeout: u64, // timeout for interchain transactions in seconds
}

#[bond_provider]
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
    PeripheralHook(Box<PuppeteerResponseHookMsg>),
    SetPause(bool),
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
    #[returns(crate::state::native_bond_provider::TxState)]
    TxState {},
    #[returns(LastPuppeteerResponse)]
    LastPuppeteerResponse {},
    #[returns(bool)]
    Pause {},
}

#[cw_serde]
pub struct MigrateMsg {}
