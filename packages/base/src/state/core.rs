use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use lido_helpers::fsm::{Fsm, Transition};

#[cw_serde]
pub struct Config {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub puppeteer_timeout: u64, //seconds
    pub strategy_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub validator_set_contract: String,
    pub owner: String,
    pub base_denom: String,
    pub ld_denom: Option<String>,
    pub idle_min_interval: u64,     //seconds
    pub unbonding_period: u64,      //seconds
    pub unbonding_safe_period: u64, //seconds
    pub pump_address: Option<String>,
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
    Withdrawn,
}

#[cw_serde]
pub struct UnbondBatch {
    pub total_amount: Uint128,
    pub expected_amount: Uint128,
    pub expected_release: u64,
    pub unbond_items: Vec<UnbondItem>,
    pub status: UnbondBatchStatus,
    pub slashing_effect: Option<Decimal>,
    pub unbonded_amount: Option<Uint128>,
    pub withdrawed_amount: Option<Uint128>,
}

pub const UNBOND_BATCHES: Map<u128, UnbondBatch> = Map::new("batches");
pub const UNBOND_BATCH_ID: Item<u128> = Item::new("batches_ids");

#[cw_serde]
pub enum ContractState {
    Idle,
    Claiming,
    Unbonding,
    Staking,
}

pub fn get_transitions() -> Vec<Transition<ContractState>> {
    vec![
        Transition {
            from: ContractState::Idle,
            to: ContractState::Claiming,
        },
        Transition {
            from: ContractState::Claiming,
            to: ContractState::Unbonding,
        },
        Transition {
            from: ContractState::Unbonding,
            to: ContractState::Staking,
        },
        Transition {
            from: ContractState::Staking,
            to: ContractState::Idle,
        },
    ]
}

pub const FSM: Item<Fsm<ContractState>> = Item::new("machine_state");
pub const LAST_IDLE_CALL: Item<u64> = Item::new("last_tick");
