use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub core_address: String,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint { amount: Uint128, receiver: String },
    Burn {},
}
#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub subdenom: String,
}

#[cosmwasm_schema::cw_serde]
pub enum MigrateMsg {}
