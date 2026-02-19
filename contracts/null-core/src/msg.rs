use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;
use drop_puppeteer_base::peripheral_hook::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    PeripheralHook(Box<PuppeteerResponseHookMsg>),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint64)]
    TotalSavedMessages {},
    #[returns((String, PuppeteerResponseHookMsg))]
    SavedMessage { id: Uint64 },
}

#[cw_serde]
pub struct MigrateMsg {}
