use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
pub use cw721::msg::Cw721InstantiateMsg as CW721InstantiateMsg;
pub use cw721::msg::MinterResponse;

use crate::state::withdrawal_voucher::{NftExtension, NftExtensionMsg};

pub type Extension = Option<NftExtension>;
pub type ExtensionMsg = Option<NftExtensionMsg>;
pub type InstantiateMsg = CW721InstantiateMsg<Empty>;
pub type ExecuteMsg = cw721::msg::Cw721ExecuteMsg<ExtensionMsg, Empty, Empty>;
pub type QueryMsg = cw721::msg::Cw721QueryMsg<Extension, Empty, Empty>;

#[cw_serde]
pub struct MigrateMsg {}
