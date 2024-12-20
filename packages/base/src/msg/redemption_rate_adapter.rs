#[allow(unused_imports)]
use crate::state::redemption_rate_adapter::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: UpdateConfig },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(RedemptionRateResponse)]
    RedemptionRate {
        denom: String,
        params: Option<Binary>,
    },
}

#[cw_serde]
pub struct RedemptionRateResponse {
    pub redemption_rate: Decimal,
    pub update_time: u64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub core_contract: String,
    pub denom: String,
}

#[cw_serde]
pub struct UpdateConfig {
    pub denom: String,
    pub core_contract: String,
}

#[cw_serde]
pub struct MigrateMsg {}
