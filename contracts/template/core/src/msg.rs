use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateExchangeRate { exchange_rate: Decimal },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Decimal)]
    ExchangeRate {},
}
