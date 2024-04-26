use astroport::router::SwapOperation;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        core_contract: Option<String>,
        price_provider_contract: Option<String>,
        cron_address: Option<String>,
        router_contract: Option<String>,
        pair_contract: Option<String>,
        from_denom: Option<String>,
        min_rewards: Option<Uint128>,
        max_spread: Option<Decimal>,
    },
    UpdateSwapOperations {
        operations: Option<Vec<SwapOperation>>,
    },
    Exchange {},
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub price_provider_contract: String,
    pub core_contract: String,
    pub cron_address: String,
    pub router_contract: String,
    pub pair_contract: String,
    pub from_denom: String,
    pub min_rewards: Uint128,
    pub swap_operations: Option<Vec<SwapOperation>>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub core_contract: String,
    pub cron_address: String,
    pub router_contract: String,
    pub price_provider_contract: String,
    pub pair_contract: String,
    pub from_denom: String,
    pub min_rewards: Uint128,
    pub max_spread: Decimal,
}

#[cw_serde]
pub enum MigrateMsg {}
