use crate::state::splitter::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Distribute {},
    UpdateConfig { new_config: Config },
}

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
