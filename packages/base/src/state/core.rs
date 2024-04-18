use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use drop_helpers::fsm::{Fsm, Transition};
use drop_puppeteer_base::msg::ResponseHookMsg;
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
#[derive(Default)]
pub struct Config {
    pub token_contract: String,
    pub puppeteer_contract: String,
    pub puppeteer_timeout: u64, //seconds
    pub strategy_contract: String,
    pub staker_contract: String,
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
    pub channel: String,
    pub ld_denom: Option<String>,
    pub lsm_min_bond_amount: Uint128,
    pub lsm_redeem_threshold: u64,
    pub lsm_redeem_maximum_interval: u64, //seconds
    pub bond_limit: Option<Uint128>,
    pub fee: Option<Decimal>,
    pub fee_address: Option<String>,
    pub emergency_address: Option<String>,
    pub min_stake_amount: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct UnbondItem {
    pub sender: String,
    pub amount: Uint128,
    pub expected_amount: Uint128,
}

#[cw_serde]
#[derive(Copy)]
pub enum UnbondBatchStatus {
    New,
    UnbondRequested,
    UnbondFailed,
    Unbonding,
    Withdrawing,
    Withdrawn,
    WithdrawingEmergency,
    WithdrawnEmergency,
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

pub struct UnbondBatchIndexes<'a> {
    pub status: MultiIndex<'a, u8, UnbondBatch, u128>,
}

impl<'a> IndexList<UnbondBatch> for UnbondBatchIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<UnbondBatch>> + '_> {
        let v: Vec<&dyn Index<UnbondBatch>> = vec![&self.status];
        Box::new(v.into_iter())
    }
}

pub fn unbond_batches_map<'a>() -> IndexedMap<'a, u128, UnbondBatch, UnbondBatchIndexes<'a>> {
    IndexedMap::new(
        "batches",
        UnbondBatchIndexes {
            status: MultiIndex::new(|_pk, b| b.status as u8, "batches", "batches__status"),
        },
    )
}

pub const UNBOND_BATCH_ID: Item<u128> = Item::new("batches_ids");
pub const TOTAL_LSM_SHARES: Item<u128> = Item::new("total_lsm_shares");
pub const PENDING_LSM_SHARES: Map<String, (String, Uint128)> = Map::new("pending_lsm_shares"); // (local_denom, (remote_denom, amount))
pub const LSM_SHARES_TO_REDEEM: Map<String, (String, Uint128)> = Map::new("lsm_shares_to_redeem");

#[cw_serde]
pub enum ContractState {
    Idle,
    Claiming,
    Unbonding,
    Staking,
    Transfering,
}

const TRANSITIONS: &[Transition<ContractState>] = &[
    Transition {
        from: ContractState::Idle,
        to: ContractState::Claiming,
    },
    Transition {
        from: ContractState::Idle,
        to: ContractState::Unbonding,
    },
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
        from: ContractState::Claiming,
        to: ContractState::Transfering,
    },
    Transition {
        from: ContractState::Transfering,
        to: ContractState::Staking,
    },
    Transition {
        from: ContractState::Transfering,
        to: ContractState::Unbonding,
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
        from: ContractState::Claiming,
        to: ContractState::Unbonding,
    },
    Transition {
        from: ContractState::Staking,
        to: ContractState::Idle,
    },
    Transition {
        from: ContractState::Unbonding,
        to: ContractState::Idle,
    },
    Transition {
        from: ContractState::Transfering,
        to: ContractState::Idle,
    },
    Transition {
        from: ContractState::Claiming,
        to: ContractState::Idle,
    },
];

#[cw_serde]
pub struct NonNativeRewardsItem {
    pub denom: String,
    pub address: String,
    pub min_amount: Uint128,
    pub fee_address: String,
    pub fee: Decimal,
}

pub const FSM: Fsm<ContractState> = Fsm::new("machine_state", TRANSITIONS);
pub const LAST_IDLE_CALL: Item<u64> = Item::new("last_tick");
pub const LAST_ICA_BALANCE_CHANGE_HEIGHT: Item<u64> = Item::new("last_ica_balance_change_height");
pub const LAST_PUPPETEER_RESPONSE: Item<ResponseHookMsg> = Item::new("last_puppeteer_response");
pub const FAILED_BATCH_ID: Item<u128> = Item::new("failed_batch_id");
pub const PRE_UNBONDING_BALANCE: Item<Uint128> = Item::new("pre_unbonding_balance");
pub const PENDING_TRANSFER: Item<Uint128> = Item::new("pending_transfer");
// Vec<(denom, address for pumping)>
pub const NON_NATIVE_REWARDS_CONFIG: Item<Vec<NonNativeRewardsItem>> =
    Item::new("non_native_rewards_config");
pub const BONDED_AMOUNT: Item<Uint128> = Item::new("bonded_amount");
pub const LAST_LSM_REDEEM: Item<u64> = Item::new("last_lsm_redeem");
pub const EXCHANGE_RATE: Item<(Decimal, u64)> = Item::new("exchange_rate");
