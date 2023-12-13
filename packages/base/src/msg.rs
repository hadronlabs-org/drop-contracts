use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum TokenExecuteMsg {
    Mint { amount: Uint128, receiver: String },
    Burn {},
}
#[cw_serde]
pub struct TokenInstantiateMsg {
    pub core_address: String,
    pub subdenom: String,
}

#[cw_serde]
pub struct CoreInstantiateMsg {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub owner: String,
}
