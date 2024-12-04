use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub release_at: u64,
    pub amount: Uint128,
    pub recipient: String,
}
