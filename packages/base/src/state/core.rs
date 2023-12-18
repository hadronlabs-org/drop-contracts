use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub voucher_contract: String,
    pub owner: String,
    pub ld_denom: Option<String>,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct UnbondItem {
    pub sender: String,
    pub amount: Uint128,
}

#[cw_serde]
pub enum UnbondBatchStatus {
    New,
    Unbonding,
    Unbonded,
}

#[cw_serde]
pub struct UnbondBatch {
    pub total_amount: Uint128,
    pub unbond_items: Vec<UnbondItem>,
    pub status: UnbondBatchStatus,
}

pub const UNBOND_BATCHES: Map<u128, UnbondBatch> = Map::new("batches");
pub const UNBOND_BATCH_ID: Item<u128> = Item::new("batches_ids");
