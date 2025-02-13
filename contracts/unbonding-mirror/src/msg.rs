use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub core_contract: String,
    pub source_port: String,
    pub source_channel: String,
    pub ibc_timeout: u64,
    pub prefix: String,
    pub retry_limit: u64,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}
