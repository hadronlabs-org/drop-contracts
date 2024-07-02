use crate::msg::staker::ResponseHookMsg as StakerResponseHookMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use drop_helpers::fsm::{Fsm, Transition};
use drop_puppeteer_base::msg::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_serde]
pub struct ConfigOptional {
    pub token_contract: Option<String>,
    pub puppeteer_contract: Option<String>,
    pub strategy_contract: Option<String>,
    pub staker_contract: Option<String>,
    pub withdrawal_voucher_contract: Option<String>,
    pub withdrawal_manager_contract: Option<String>,
    pub validators_set_contract: Option<String>,
    pub base_denom: Option<String>,
    pub remote_denom: Option<String>,
    pub idle_min_interval: Option<u64>,
    pub unbonding_period: Option<u64>,
    pub unbonding_safe_period: Option<u64>,
    pub unbond_batch_switch_time: Option<u64>,
    pub pump_ica_address: Option<String>,
    pub transfer_channel_id: Option<String>,
    pub lsm_min_bond_amount: Option<Uint128>,
    pub lsm_redeem_threshold: Option<u64>,
    pub lsm_redeem_maximum_interval: Option<u64>,
    pub bond_limit: Option<Uint128>,
    pub fee: Option<Decimal>,
    pub fee_address: Option<String>,
    pub emergency_address: Option<String>,
    pub min_stake_amount: Option<Uint128>,
}

#[cw_serde]
pub struct Config {
    pub token_contract: Addr,
    pub puppeteer_contract: Addr,
    pub strategy_contract: Addr,
    pub staker_contract: Addr,
    pub withdrawal_voucher_contract: Addr,
    pub withdrawal_manager_contract: Addr,
    pub validators_set_contract: Addr,
    pub base_denom: String,
    pub remote_denom: String,
    pub idle_min_interval: u64,        //seconds
    pub unbonding_period: u64,         //seconds
    pub unbonding_safe_period: u64,    //seconds
    pub unbond_batch_switch_time: u64, //seconds
    pub pump_ica_address: Option<String>,
    pub transfer_channel_id: String,
    pub lsm_min_bond_amount: Uint128,
    pub lsm_redeem_threshold: u64,
    pub lsm_redeem_maximum_interval: u64, //seconds
    pub bond_limit: Option<Uint128>,
    pub fee: Option<Decimal>,
    pub fee_address: Option<String>,
    pub emergency_address: Option<String>,
    pub min_stake_amount: Uint128,
    pub icq_update_delay: u64, // blocks
}

pub const CONFIG: Item<Config> = Item::new("config");

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
pub struct UnbondBatchStatusTimestamps {
    pub new: u64,
    pub unbond_requested: Option<u64>,
    pub unbond_failed: Option<u64>,
    pub unbonding: Option<u64>,
    pub withdrawing: Option<u64>,
    pub withdrawn: Option<u64>,
    pub withdrawing_emergency: Option<u64>,
    pub withdrawn_emergency: Option<u64>,
}

#[cw_serde]
pub struct UnbondBatch {
    pub total_dasset_amount_to_withdraw: Uint128,
    pub expected_native_asset_amount: Uint128,
    pub expected_release_time: u64,
    pub total_unbond_items: u64,
    pub status: UnbondBatchStatus,
    pub slashing_effect: Option<Decimal>,
    pub unbonded_amount: Option<Uint128>,
    pub withdrawed_amount: Option<Uint128>,
    pub status_timestamps: UnbondBatchStatusTimestamps,
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
    LSMTransfer,
    LSMRedeem,
    NonNativeRewardsTransfer,
    Claiming,
    Unbonding,
    StakingRewards,
    StakingBond,
}

const TRANSITIONS: &[Transition<ContractState>] = &[
    Transition {
        from: ContractState::Idle,
        to: ContractState::LSMTransfer,
    },
    Transition {
        from: ContractState::Idle,
        to: ContractState::LSMRedeem,
    },
    Transition {
        from: ContractState::Idle,
        to: ContractState::NonNativeRewardsTransfer,
    },
    Transition {
        from: ContractState::LSMTransfer,
        to: ContractState::Idle,
    },
    Transition {
        from: ContractState::LSMRedeem,
        to: ContractState::Idle,
    },
    Transition {
        from: ContractState::NonNativeRewardsTransfer,
        to: ContractState::Idle,
    },
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
        to: ContractState::StakingRewards,
    },
    Transition {
        from: ContractState::Idle,
        to: ContractState::StakingBond,
    },
    Transition {
        from: ContractState::Claiming,
        to: ContractState::StakingBond,
    },
    Transition {
        from: ContractState::StakingBond,
        to: ContractState::StakingRewards,
    },
    Transition {
        from: ContractState::StakingBond,
        to: ContractState::Unbonding,
    },
    Transition {
        from: ContractState::StakingRewards,
        to: ContractState::Unbonding,
    },
    Transition {
        from: ContractState::Claiming,
        to: ContractState::StakingRewards,
    },
    Transition {
        from: ContractState::Claiming,
        to: ContractState::Unbonding,
    },
    Transition {
        from: ContractState::StakingRewards,
        to: ContractState::Idle,
    },
    Transition {
        from: ContractState::Unbonding,
        to: ContractState::Idle,
    },
    Transition {
        from: ContractState::StakingBond,
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
pub const LAST_ICA_CHANGE_HEIGHT: Item<u64> = Item::new("last_ica_change_height");
pub const LAST_PUPPETEER_RESPONSE: Item<PuppeteerResponseHookMsg> =
    Item::new("last_puppeteer_response");
pub const LAST_STAKER_RESPONSE: Item<StakerResponseHookMsg> = Item::new("last_staker_response");
pub const FAILED_BATCH_ID: Item<u128> = Item::new("failed_batch_id");
// Vec<(denom, address for pumping)>
pub const NON_NATIVE_REWARDS_CONFIG: Item<Vec<NonNativeRewardsItem>> =
    Item::new("non_native_rewards_config");
pub const BONDED_AMOUNT: Item<Uint128> = Item::new("bonded_amount"); // to be used in bond limit
pub const LAST_LSM_REDEEM: Item<u64> = Item::new("last_lsm_redeem");
pub const EXCHANGE_RATE: Item<(Decimal, u64)> = Item::new("exchange_rate");
pub const LD_DENOM: Item<String> = Item::new("ld_denom");
