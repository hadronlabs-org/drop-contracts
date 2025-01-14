use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetRewardsAddress { address: String },
    ClaimRewards {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<cosmwasm_std::Coin>)]
    PendingRewards { address: String },
}

#[cw_serde]
pub struct MigrateMsg {}
