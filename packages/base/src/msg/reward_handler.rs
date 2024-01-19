use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub enum HandlerExecuteMsg {
    Exchange { coin: Coin },
}

