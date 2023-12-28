use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        core_address: Option<String>,
        puppeteer_address: Option<String>,
        validator_set_address: Option<String>,
        distribution_address: Option<String>,
        denom: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Vec<super::distribution::IdealDelegation>)]
    CalcDeposit { deposit: Uint128 },
    #[returns(Vec<super::distribution::IdealDelegation>)]
    CalcWithdraw { withdraw: Uint128 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub core_address: String,
    pub puppeteer_address: String,
    pub validator_set_address: String,
    pub distribution_address: String,
    pub denom: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub puppeteer_address: String,
    pub validator_set_address: String,
    pub distribution_address: String,
    pub denom: String,
}

#[cw_serde]
pub enum MigrateMsg {}
