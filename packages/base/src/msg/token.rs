use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ExecuteMsg {
    Mint { amount: Uint128, receiver: String },
    Burn {},
}
#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub subdenom: String,
}

#[cosmwasm_schema::cw_serde]
pub enum MigrateMsg {}
