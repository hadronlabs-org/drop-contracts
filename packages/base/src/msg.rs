use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum TokenExecuteMsg {
    Mint { amount: Uint128, receiver: String },
    Burn {},
}
