use crate::state::ChainDetails;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct AddChain {
    pub name: String,
    pub details: ChainDetails,
}

#[cw_serde]
pub struct RemoveChainList {
    pub names: Vec<String>,
}

#[cw_serde]
pub struct AddChainList {
    pub chains: Vec<AddChain>,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    AddChains(AddChainList),
    RemoveChains(RemoveChainList),
}

#[cw_serde]
pub struct ChainInfo {
    pub name: String,
    pub details: ChainDetails,
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ChainInfo)]
    Chain { name: String },
    #[returns(Vec<ChainInfo>)]
    Chains {},
}
