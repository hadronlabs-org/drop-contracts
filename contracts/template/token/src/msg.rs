use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DenomMetadata, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub token_metadata: DenomMetadata,
    pub subdenom: String,
    pub exponent: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint { amount: Uint128 },
    Burn {},
}

#[cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    Denom {},
}

#[cw_serde]
pub struct MigrateMsg {}
