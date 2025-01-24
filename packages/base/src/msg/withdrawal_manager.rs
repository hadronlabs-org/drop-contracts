use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw721::Cw721ReceiveMsg;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
#[allow(unused_imports)]
use drop_helpers::pause::PauseInfoResponse;
use drop_macros::{pausable, pausable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_contract: String,
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
        factory_contract: Option<String>,
        base_denom: Option<String>,
    },
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum ReceiveNftMsg {
    Withdraw { receiver: Option<String> },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct WithdrawalVoucherTrait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
#[derive(Default)]
pub struct WithdrawalVoucherMetadata {
    pub name: String,
    pub description: Option<String>,
    pub attributes: Option<Vec<WithdrawalVoucherTrait>>,
    pub batch_id: String,
    pub amount: Uint128,
}

pub type WithdrawalVoucherExtension = Option<WithdrawalVoucherMetadata>;

#[cw_serde]
pub struct WithdrawalVoucherNftInfoMsg {
    pub token_id: String
}
