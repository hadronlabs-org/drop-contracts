use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub enum ExecuteMsg {
    Exchange { coin: Coin },
}
