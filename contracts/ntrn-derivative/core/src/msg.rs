use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::DenomMetadata;
use cw_ownable::cw_ownable_execute;

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Bond { receiver: Option<String> },
    Unbond { receiver: Option<String> },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub withdrawal_voucher_code_id: u64,
    pub unbonding_period: u64,
    pub token_metadata: DenomMetadata,
    pub subdenom: String,
    pub exponent: u32,
}

#[cw_serde]
pub struct MigrateMsg {}
