use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cosmwasm_schema::cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub core_address: String,
    pub denom: String,
}

pub const CORE_ADDRESS: Item<Addr> = Item::new("core");
pub const DENOM: Item<String> = Item::new("denom");
