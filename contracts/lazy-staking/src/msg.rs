use crate::state::Config;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::DenomMetadata;

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
    #[returns(cosmwasm_std::Decimal)]
    ExchangeRate {},
    #[returns(cosmwasm_std::Decimal)]
    Rewards {},
    #[returns(String)]
    Denom {},
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Bond {},
    Unbond {},
    WithdrawRewards {},
}

#[cw_serde]
pub struct MigrateMsg {}