use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};
use drop_macros::{bond_provider, bond_provider_query};

#[bond_provider_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[bond_provider]
#[cw_serde]
pub enum ExecuteMsg {}
