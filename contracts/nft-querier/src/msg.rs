use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::cw_ownable_execute;

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_contract: String,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: Config },
}

#[cw_serde]
pub enum NftState {
    Ready,
    NotReady,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    Config {},
    #[returns(NftState)]
    NftState { nft_id: String },
}
