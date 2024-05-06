use crate::state::ChainInfo;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct AddChainInfo {
    pub name: String,
    pub details: ChainInfo,
}

#[cw_serde]
pub struct AddChainInfoList {
    pub chains: Vec<AddChainInfo>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddChainsInfo(AddChainInfoList),
}

#[cw_serde]
pub struct ChainInfoReponse {
    pub name: String,
    pub details: ChainInfo,
}

#[cw_serde]
pub enum QueryMsg {
    ChainInfo { name: String },
    ChainsInfo {},
}
