use astroport::router::SwapOperation;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        core_contract: Option<String>,
        cron_address: Option<String>,
        router_contract: Option<String>,
        swap_contract: Option<String>,
        from_denom: Option<String>,
        min_rewards: Option<Uint128>,
    },
    UpdateSwapOperations {
        operations: Option<Vec<SwapOperation>>,
    },
    Exchange {
        coin: Coin,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub core_contract: String,
    pub cron_address: String,
    pub router_contract: String,
    pub swap_contract: String,
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
    pub swap_contract: String,
    pub from_denom: String,
    pub min_rewards: Uint128,
}

#[cw_serde]
pub enum MigrateMsg {}
