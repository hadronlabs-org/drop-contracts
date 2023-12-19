use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::State;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_code_id: u64,
    pub core_code_id: u64,
    pub voucher_code_id: u64,
    pub salt: String,
    pub subdenom: String,
}

#[cw_serde]
pub enum CallbackMsg {
    PostInit {},
}

#[cw_serde]
pub enum ExecuteMsg {
    Init { base_denom: String },
    Callback(CallbackMsg),
}
#[cw_serde]
pub enum MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(State)]
    State {},
}
