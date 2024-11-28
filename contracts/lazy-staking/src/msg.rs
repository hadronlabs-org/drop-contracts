use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, DenomMetadata};

#[cw_serde]
pub struct InstantiateMsg {
    pub config: Config,
    pub token_metadata: DenomMetadata,
    pub subdenom: String,
    pub exponent: u32,
}

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Decimal)]
    ExchangeRate,
    #[returns(Decimal)]
    Rewards,
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Bond,
    Unbond,
    WithdrawRewards,
}

#[cw_serde]
pub struct MigrateMsg {}
