use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

pub const FACTORY_CONTRACT: Item<Addr> = Item::new("factory_contract");
pub const LD_TOKEN: Item<String> = Item::new("ld_token");

pub use bondings::{map as bondings_map, BondingRecord};
mod bondings {
    use super::*;

    #[cw_serde]
    pub struct BondingRecord {
        pub bonder: Addr,
        pub deposit: Vec<Coin>,
    }

    pub struct BondingRecordIndexes<'a> {
        pub bonder: MultiIndex<'a, Addr, BondingRecord, &'a str>,
    }

    impl<'a> IndexList<BondingRecord> for BondingRecordIndexes<'a> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<BondingRecord>> + '_> {
            let v: Vec<&dyn Index<BondingRecord>> = vec![&self.bonder];
            Box::new(v.into_iter())
        }
    }

    pub fn map<'a>() -> IndexedMap<'a, &'a str, BondingRecord, BondingRecordIndexes<'a>> {
        IndexedMap::new(
            "bondings",
            BondingRecordIndexes {
                bonder: MultiIndex::new(|_pk, b| b.bonder.clone(), "bondings", "bondings__bonder"),
            },
        )
    }
}

pub mod reply {
    use super::*;

    #[cw_serde]
    pub struct CoreUnbond {
        pub sender: Addr,
        pub deposit: Vec<Coin>,
    }
    pub const CORE_UNBOND: Item<CoreUnbond> = Item::new("reply_core_unbond");
}
