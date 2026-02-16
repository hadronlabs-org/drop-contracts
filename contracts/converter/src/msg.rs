use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub rate: Decimal,
    pub from_token: String,
    pub to_token: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Swap { receiver: String },
    UpdateRate { rate: Decimal },
    Clawback {},
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    Config {},
    #[returns(Decimal)]
    Rate(),
}

#[cw_serde]
pub struct MigrateMsg {}
