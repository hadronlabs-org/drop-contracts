use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub strategy_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub owner: String,
    pub base_denom: String,
    pub ld_denom: Option<String>,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct UnbondItem {
    pub sender: String,
    pub amount: Uint128,
    pub expected_amount: Uint128,
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
    pub expected_amount: Uint128,
    pub unbond_items: Vec<UnbondItem>,
    pub status: UnbondBatchStatus,
    pub slashing_effect: Option<Decimal>,
    pub unbonded_amount: Option<Uint128>,
    pub withdrawed_amount: Option<Uint128>,
}

pub const UNBOND_BATCHES: Map<u128, UnbondBatch> = Map::new("batches");
pub const UNBOND_BATCH_ID: Item<u128> = Item::new("batches_ids");
