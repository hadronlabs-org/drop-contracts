use crate::state::token::Pause;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Pause)]
    Pause {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub factory_contract: String,
    pub denom: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Mint { amount: Uint128, receiver: String },
    Burn {},
    SetTokenMetadata { token_metadata: DenomMetadata },
    SetPause(Pause),
}

#[cw_serde]
pub struct DenomMetadata {
    /// Number of decimals
    pub exponent: u32,
    /// Lowercase moniker to be displayed in clients, example: "atom"
    pub display: String,
    /// Descriptive token name, example: "Cosmos Hub Atom"
    pub name: String,
    /// Even longer description, example: "The native staking token of the Cosmos Hub"
    pub description: String,
    /// Symbol to be displayed on exchanges, example: "ATOM"
    pub symbol: String,
    /// URI to a document that contains additional information
    pub uri: Option<String>,
    /// SHA256 hash of a document pointed by URI
    pub uri_hash: Option<String>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_contract: String,
    pub subdenom: String,
    pub token_metadata: DenomMetadata,
    pub owner: String,
}

#[cw_serde]
pub struct MigrateMsg {}
