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
    #[returns(Vec<super::distribution::IdealDelegation>)]
    CalcDeposit { deposit: Uint128 },
    #[returns(Vec<super::distribution::IdealDelegation>)]
    CalcWithdraw { withdraw: Uint128 },
}

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub puppeteer_address: String,
    pub validator_set_address: String,
    pub distribution_address: String,
    pub denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub puppeteer_address: String,
    pub validator_set_address: String,
    pub distribution_address: String,
    pub denom: String,
}

#[cw_serde]
pub enum MigrateMsg {}
