use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use lido_helpers::fsm::{Fsm, Transition};
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
#[derive(Default)]
pub struct Config {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub puppeteer_timeout: u64, //seconds
    pub strategy_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub validators_set_contract: String,
    pub base_denom: String,
    pub remote_denom: String,
    pub idle_min_interval: u64,        //seconds
    pub unbonding_period: u64,         //seconds
    pub unbonding_safe_period: u64,    //seconds
    pub unbond_batch_switch_time: u64, //seconds
    pub pump_address: Option<String>,
    pub owner: String,
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
    UnbondRequested,
    UnbondFailed,
    Unbonding,
    Unbonded,
    Withdrawn,
}

#[cw_serde]
pub struct UnbondBatch {
    pub total_amount: Uint128,
    pub expected_amount: Uint128,
    pub expected_release: u64,
    // TODO: this always growing array should definitely be refactored into some kind of a map,
    //       because each successfull unbond call will consume more and more gas on (de)serialization
    //       until it eventually doesn't fit in a block anymore
    pub unbond_items: Vec<UnbondItem>,
    pub status: UnbondBatchStatus,
    pub slashing_effect: Option<Decimal>,
    pub unbonded_amount: Option<Uint128>,
    pub withdrawed_amount: Option<Uint128>,
    pub created: u64,
}

pub const UNBOND_BATCHES: Map<u128, UnbondBatch> = Map::new("batches");
pub const UNBOND_BATCH_ID: Item<u128> = Item::new("batches_ids");

#[cw_serde]
pub enum ContractState {
    Idle,
    Claiming,
    Unbonding,
    Staking,
    Transfering,
}

pub fn get_transitions() -> Vec<Transition<ContractState>> {
    vec![
        Transition {
            from: ContractState::Idle,
            to: ContractState::Claiming,
        },
        Transition {
            from: ContractState::Idle,
            to: ContractState::Staking,
        },
        Transition {
            from: ContractState::Idle,
            to: ContractState::Transfering,
        },
        Transition {
            from: ContractState::Idle,
            to: ContractState::Claiming,
        },
        Transition {
            from: ContractState::Claiming,
            to: ContractState::Transfering,
        },
        Transition {
            from: ContractState::Transfering,
            to: ContractState::Staking,
        },
        Transition {
            from: ContractState::Staking,
            to: ContractState::Unbonding,
        },
        Transition {
            from: ContractState::Claiming,
            to: ContractState::Staking,
        },
        Transition {
            from: ContractState::Staking,
            to: ContractState::Idle,
        },
        Transition {
            from: ContractState::Unbonding,
            to: ContractState::Idle,
        },
    ]
}

pub const FSM: Item<Fsm<ContractState>> = Item::new("machine_state");
pub const LAST_IDLE_CALL: Item<u64> = Item::new("last_tick");
pub const LAST_ICA_BALANCE_CHANGE_HEIGHT: Item<u64> = Item::new("last_ica_balance_change_height");
pub const LAST_PUPPETEER_RESPONSE: Item<lido_puppeteer_base::msg::ResponseHookMsg> =
    Item::new("last_puppeteer_response");
pub const FAILED_BATCH_ID: Item<u128> = Item::new("failed_batch_id");
pub const PRE_UNBONDING_BALANCE: Item<Uint128> = Item::new("pre_unbonding_balance");
