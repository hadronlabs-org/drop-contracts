use crate::state::{withdrawal_voucher::Metadata, withdrawal_voucher::Pause};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
pub use cw721_base::{ContractError, InstantiateMsg as CW721InstantiateMsg, MinterResponse};

pub type Extension = Option<Metadata>;
pub type InstantiateMsg = CW721InstantiateMsg;
pub type CW721ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
#[cw_serde]
pub enum ExecuteMsg {
    Custom { msg: CW721ExecuteMsg },
    SetPause(Pause),
}
pub type QueryMsg = cw721_base::QueryMsg<Empty>;

#[cw_serde]
pub struct MigrateMsg {}
