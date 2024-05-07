use crate::state::FactoryInstance;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct DropInstance {
    pub name: String,
    pub details: FactoryInstance,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    AddChains { chains: Vec<DropInstance> },
    RemoveChains { names: Vec<String> },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DropInstance)]
    Chain { name: String },
    #[returns(Vec<DropInstance>)]
    Chains {},
}
