use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
pub use cw721::msg::Cw721InstantiateMsg as CW721InstantiateMsg;
pub use cw721::msg::MinterResponse;
pub use cw721_base::error::ContractError;

use crate::state::withdrawal_voucher::Metadata;

pub type Extension = Option<Metadata>;
pub type InstantiateMsg = CW721InstantiateMsg<Empty>;
pub type ExecuteMsg = cw721::msg::Cw721ExecuteMsg<Extension, Empty, Empty>;
pub type QueryMsg = cw721::msg::Cw721QueryMsg<Empty, Empty, Empty>;

#[cw_serde]
pub struct MigrateMsg {}
