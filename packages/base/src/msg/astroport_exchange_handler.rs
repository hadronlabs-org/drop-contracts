use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { core_address: Option<String> },
    Exchange { coin: Coin },
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
}

#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
}

#[cw_serde]
pub enum MigrateMsg {}
