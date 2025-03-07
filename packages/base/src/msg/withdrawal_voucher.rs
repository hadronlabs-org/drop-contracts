use cosmwasm_schema::cw_serde;
pub use cw721::msg::MinterResponse;
pub use cw721_base::error::ContractError;
pub use cw721_base::msg::InstantiateMsg as CW721InstantiateMsg;

use crate::state::withdrawal_voucher::Metadata;

pub type Extension = Option<Metadata>;
pub type InstantiateMsg = CW721InstantiateMsg;
pub type ExecuteMsg = cw721_base::msg::ExecuteMsg;
pub type QueryMsg = cw721_base::msg::QueryMsg;

#[cw_serde]
pub struct MigrateMsg {}
