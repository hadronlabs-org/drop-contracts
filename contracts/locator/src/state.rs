use cosmwasm_schema::cw_serde;
use cw_storage_plus::Map;
use drop_staking_base::state::factory::State as FactoryState;

#[cw_serde]
pub struct FactoryInstance {
    addr: String,
    contracts: FactoryState,
}

pub const STATE: Map<String, FactoryInstance> = Map::new("state");
