use crate::state::ChainInfo;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct AddChainInfo {
    pub key: String,
    pub chain_info: ChainInfo,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddChainInfo(AddChainInfo),
}

#[cw_serde]
// #[derive(QueryResponses)]
pub enum QueryMsg {
    ChainInfo { chain: String },
    // #[returns(crate::state::STATE)]
    // ChainsInfo {},
}
