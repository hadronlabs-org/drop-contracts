use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use optfield::optfield;

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(Vec<(String, Uint128)>)]
    CalcDeposit { deposit: Uint128 },
    #[returns(Vec<(String, Uint128)>)]
    CalcWithdraw { withdraw: Uint128 },
}

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub factory_contract: String,
    pub denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub factory_contract: String,
    pub denom: String,
}

#[cw_serde]
pub struct MigrateMsg {
    pub factory_contract: String,
}
