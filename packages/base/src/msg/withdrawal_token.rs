use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub core_address: String,
    pub withdrawal_manager_address: String,
    pub denom_prefix: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    CreateDenom {
        batch_id: Uint128,
    },
    Mint {
        amount: Uint128,
        receiver: String,
        batch_id: Uint128,
    },
    Burn {
        batch_id: Uint128,
    },
    Premint {},
    DisableInitState {},
}
#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub withdrawal_manager_address: String,
    pub withdrawal_exchange_address: String,
    pub denom_prefix: String,
    pub is_init_state: bool,
    pub owner: String,
}

#[cw_serde]
pub struct MigrateMsg {}
