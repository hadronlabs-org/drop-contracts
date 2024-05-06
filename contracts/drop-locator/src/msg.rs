use crate::state::ChainInfo;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct AddChainInfoResponse {
    pub name: String,
    pub chain_info: ChainInfo,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddChainInfo(AddChainInfoResponse),
}

#[cw_serde]
pub struct ChainInfoReponse {
    pub name: String,
    pub chain_info: ChainInfo,
}

#[cw_serde]
pub enum QueryMsg {
    ChainInfo { name: String },
    ChainsInfo {},
}
