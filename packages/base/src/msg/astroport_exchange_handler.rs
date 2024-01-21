use astroport::router::SwapOperation;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        core_address: Option<String>,
        cron_address: Option<String>,
        router_contract_address: Option<String>,
        from_denom: Option<String>,
    },
    SetRouteAndSwap {
        operations: Vec<SwapOperation>,
    },
    DirectSwap {
        contract_address: String,
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
    pub core_address: String,
    pub cron_address: String,
    pub router_contract_address: String,
    pub from_denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub cron_address: String,
    pub router_contract_address: String,
    pub from_denom: String,
}

#[cw_serde]
pub enum MigrateMsg {}
