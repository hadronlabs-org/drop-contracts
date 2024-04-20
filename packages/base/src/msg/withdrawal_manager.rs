use cosmwasm_schema::{cw_serde, QueryResponses};
use cw721::Cw721ReceiveMsg;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
#[allow(unused_imports)]
use drop_helpers::pause::PauseInfoResponse;
use drop_macros::{pausable, pausable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub core_contract: String,
    pub voucher_contract: String,
    pub base_denom: String,
    pub owner: String,
}

#[cw_ownable_query]
#[pausable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::withdrawal_manager::Config)]
    Config {},
}

#[cw_ownable_execute]
#[pausable]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        core_contract: Option<String>,
        voucher_contract: Option<String>,
    },
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum ReceiveNftMsg {
    Withdraw { receiver: Option<String> },
}

#[cw_serde]
pub enum MigrateMsg {}
