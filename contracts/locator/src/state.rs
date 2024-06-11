use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;
#[cw_serde]
pub struct DropInstance {
    pub name: String,
    pub factory_addr: String,
}

// chain's name -> factory instance
pub const STATE: Map<String, DropInstance> = Map::new("state");
