use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub prefix: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Link { address: String },
    AdminLink { from: String, address: String },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetOne { address: String },
    #[returns(Vec<(String, String)>)]
    GetAll {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct MigrateMsg {}
