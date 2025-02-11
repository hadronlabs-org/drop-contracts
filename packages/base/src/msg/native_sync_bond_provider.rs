use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use drop_macros::{bond_provider, bond_provider_query};
use drop_puppeteer_base::peripheral_hook::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub factory_contract: String,
}

#[cw_serde]
pub struct ConfigOptional {
    pub factory_contract: Option<String>,
}

#[bond_provider]
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
    PeripheralHook(Box<PuppeteerResponseHookMsg>),
}

#[bond_provider_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::native_sync_bond_provider::Config)]
    Config {},
    #[returns(cosmwasm_std::Uint128)]
    NonStakedBalance {},
    #[returns(crate::state::native_bond_provider::TxState)]
    TxState {},
    #[returns(crate::msg::core::LastPuppeteerResponse)]
    LastPuppeteerResponse {},
}

#[cw_serde]
pub struct MigrateMsg {}
