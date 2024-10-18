use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub withdrawal_token_address: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Exchange {},
}
#[cw_serde]
pub struct InstantiateMsg {
    pub withdrawal_token_address: String,
    pub owner: String,
}

#[cw_serde]
pub struct MigrateMsg {}
