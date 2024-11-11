use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub batch_id: String,
    pub amount: Uint128,
}

#[cw_serde]
#[derive(Default)]
pub struct Pause {
    pub mint: bool,
}

pub const PAUSE: Item<Pause> = Item::new("pause");
