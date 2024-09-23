use crate::state::native_bond_provider::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
#[allow(unused_imports)]
use cosmwasm_std::{Coin, Decimal, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use drop_macros::{bond_provider, bond_provider_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub base_denom: String,
    pub staker_contract: String,
}

#[bond_provider]
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
}

#[bond_provider_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::native_bond_provider::Config)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
