use cw_storage_plus::Item;
use drop_staking_base::state::factory::State as FactoryState;

pub const STATE: Item<FactoryState> = Item::new("state");
