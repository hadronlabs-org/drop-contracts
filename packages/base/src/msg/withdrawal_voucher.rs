use crate::state::{withdrawal_voucher::Metadata, withdrawal_voucher::Pause};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::CustomMsg;

pub use cw721_base::{ContractError, InstantiateMsg as CW721InstantiateMsg, MinterResponse};

pub type Extension = Option<Metadata>;
pub type InstantiateMsg = CW721InstantiateMsg;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, ExtensionExecuteMsg>;
pub type QueryMsg = cw721_base::QueryMsg<ExtensionQueryMsg>;

#[cw_serde]
pub enum ExtensionExecuteMsg {
    SetPause(Pause),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum ExtensionQueryMsg {
    #[returns(Pause)]
    Pause,
}

impl Default for ExtensionExecuteMsg {
    fn default() -> Self {
        ExtensionExecuteMsg::SetPause(Pause { mint: false })
    }
}

impl CustomMsg for ExtensionExecuteMsg {}

impl Default for ExtensionQueryMsg {
    fn default() -> Self {
        ExtensionQueryMsg::Pause {}
    }
}

impl CustomMsg for ExtensionQueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}
