use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
pub use cw721_base::{ContractError, InstantiateMsg as CW721InstantiateMsg, MinterResponse};
use crate::state::Metadata;

pub type Extension = Option<Metadata>;
pub type InstantiateMsg = CW721InstantiateMsg;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
pub type QueryMsg = cw721_base::QueryMsg<Empty>;

#[cw_serde]
pub struct MigrateMsg {}