use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    CanBond { denom: String },
    #[returns(bool)]
    CanProcessOnIdle {},
    #[returns(Decimal)]
    TokenAmount { coin: Coin, exchange_rate: Decimal },
}

#[cw_serde]
pub enum ExecuteMsg {
    Bond {},
    ProcessOnIdle {},
}
