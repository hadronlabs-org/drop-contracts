use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::State;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_code_id: u64,
    pub core_code_id: u64,
    pub withdrawal_voucher_code_id: u64,
    pub withdrawal_manager_code_id: u64,
    pub strategy_code_id: u64,
    pub validators_set_code_id: u64,
    pub distribution_code_id: u64,
    pub salt: String,
    pub subdenom: String,
}

#[cw_serde]
pub enum CallbackMsg {
    PostInit {},
}
#[cw_serde]
pub struct CoreParams {
    pub idle_min_interval: u64,
    pub puppeteer_timeout: u64,
    pub unbonding_period: u64,
    pub unbonding_safe_period: u64,
    pub unbond_batch_switch_time: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Init {
        base_denom: String,
        core_params: CoreParams,
    },
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
