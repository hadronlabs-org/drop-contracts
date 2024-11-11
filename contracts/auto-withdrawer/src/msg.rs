use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub withdrawal_token_address: String,
    pub withdrawal_manager_address: String,
    pub ld_token: String,
    pub withdrawal_denom_prefix: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bond(BondMsg),
    Unbond {
        batch_id: Uint128,
    },
    Withdraw {
        batch_id: Uint128,
        receiver: Option<Addr>,
    },
}

#[cw_serde]
pub enum BondMsg {
    WithLdAssets {},
    WithWithdrawalDenoms { batch_id: Uint128 },
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
    pub bonding_id: String,
    pub bonder: String,
    pub deposit: Vec<Coin>,
    pub withdrawal_amount: Uint128,
}

#[cw_serde]
pub struct MigrateMsg {}
