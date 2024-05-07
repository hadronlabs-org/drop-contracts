use crate::state::DropInstance;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use drop_staking_base::state::factory::State as FactoryState;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct FactoryInstance {
    pub addr: String,
    pub contracts: FactoryState,
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
    #[returns(FactoryInstance)]
    FactoryInstance { name: String },
    #[returns(Vec<FactoryInstance>)]
    FactoryInstances {},
    #[returns(DropInstance)]
    Chain { name: String },
    #[returns(Vec<DropInstance>)]
    Chains {},
}
