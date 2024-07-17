use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

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
