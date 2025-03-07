use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_contract: String,
    pub ld_token: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bond(BondMsg),
    Unbond { token_id: String },
    Withdraw { token_id: String },
}

#[cw_serde]
pub enum BondMsg {
    WithLdAssets {},
    WithNFT { token_id: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// List all bondings
    #[returns(BondingsResponse)]
    Bondings {
        /// Optionally filter bondings by user address
        user: Option<String>,
        /// Pagination limit. Default is 100
        limit: Option<Uint64>,
        /// Pagination offset
        page_key: Option<String>,
    },
    #[returns(InstantiateMsg)] // config is static and is 100% similar to InstantiateMsg
    Config {},
}

#[cw_serde]
pub struct BondingsResponse {
    pub bondings: Vec<BondingResponse>,
    pub next_page_key: Option<String>,
}

#[cw_serde]
pub struct BondingResponse {
    pub token_id: String,
    pub bonder: String,
    pub deposit: Vec<Coin>,
}

#[cw_serde]
pub struct MigrateMsg {}
