use crate::state::withdrawal_manager::Pause;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw721::receiver::Cw721ReceiveMsg;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_contract: String,
    pub base_denom: String,
    pub owner: String,
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::withdrawal_manager::Config)]
    Config {},
    #[returns(Pause)]
    Pause {},
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        factory_contract: Option<String>,
        base_denom: Option<String>,
    },
    ReceiveNft(Cw721ReceiveMsg),
    SetPause {
        pause: Pause,
    },
}

#[cw_serde]
pub enum ReceiveNftMsg {
    Withdraw { receiver: Option<String> },
}

#[cw_serde]
pub struct MigrateMsg {}
