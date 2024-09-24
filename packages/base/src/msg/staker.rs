use crate::state::staker::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use drop_puppeteer_base::msg::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::pump::Config)]
    Config {},
    #[returns(Uint128)]
    NonStakedBalance {},
    #[returns(Uint128)]
    AllBalance {},
}

#[cw_serde]
pub struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    PuppeteerTransfer {},
    UpdateConfig { new_config: Box<ConfigOptional> },
    PuppeteerHook(Box<PuppeteerResponseHookMsg>),
}

#[cw_serde]
pub struct InstantiateMsg {
    pub remote_denom: String,
    pub base_denom: String,
    pub puppeteer_contract: String,
    pub core_contract: String,
    pub owner: Option<String>,
    pub min_ibc_transfer: Uint128,
}

#[cw_serde]
pub struct MigrateMsg {}
