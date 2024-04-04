use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, UniqueIndex};
use drop_helpers::ica::Ica;
use neutron_sdk::{bindings::msg::IbcFee, interchain_queries::v045::types::UnbondingEntry};
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::Transaction;

pub struct PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub config: Item<'a, T>,
    pub ica: Ica<'a>,
    pub recipient_transfers: Item<'a, Vec<Transfer>>,
    pub transfer_channel_id: Item<'a, String>,
    pub tx_state: Item<'a, TxState>,
    pub ibc_fee: Item<'a, IbcFee>,
    pub register_fee: Item<'a, Coin>,
    pub kv_queries: Map<'a, u64, U>,
    pub unbonding_delegations:
        IndexedMap<'a, &'a str, UnbondingDelegation, UnbondingDelegationIndexes<'a>>,
    pub unbonding_delegations_reply_id_storage: Map<'a, u16, UnbondingDelegation>,
}

impl<T, U> Default for PuppeteerBase<'static, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T, U> PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub fn new() -> Self {
        Self {
            config: Item::new("config"),
            ica: Ica::new("ica"),
            recipient_transfers: Item::new("transfers"),
            tx_state: Item::new("sudo_payload"),
            ibc_fee: Item::new("ibc_fee"),
            register_fee: Item::new("register_fee"),
            transfer_channel_id: Item::new("transfer_channel_id"),
            kv_queries: Map::new("kv_queries"),
            unbonding_delegations: IndexedMap::new(
                "unbonding_delegations",
                UnbondingDelegationIndexes {
                    query_id: UniqueIndex::new(
                        |d: &UnbondingDelegation| d.query_id,
                        "unbonding_delegations__query_id",
                    ),
                },
            ),
            unbonding_delegations_reply_id_storage: Map::new(
                "unbonding_delegations_reply_id_storage",
            ),
        }
    }
}

pub trait BaseConfig {
    fn connection_id(&self) -> String;
    fn update_period(&self) -> u64;
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub connection_id: String,
    pub update_period: u64,
}

#[cw_serde]
pub struct Transfer {
    pub recipient: String,
    pub sender: String,
    pub denom: String,
    pub amount: String,
}

#[cw_serde]
#[derive(Default)]
pub enum TxStateStatus {
    #[default]
    Idle,
    InProgress,
    WaitingForAck,
}

impl std::fmt::Display for TxStateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxStateStatus::Idle => write!(f, "Idle"),
            TxStateStatus::InProgress => write!(f, "InProgress"),
            TxStateStatus::WaitingForAck => write!(f, "WaitingForAck"),
        }
    }
}

#[cw_serde]
#[derive(Default)]
pub struct TxState {
    pub status: TxStateStatus,
    pub seq_id: Option<u64>,
    pub transaction: Option<Transaction>,
    pub reply_to: Option<String>,
}

#[cw_serde]
pub struct UnbondingDelegation {
    pub validator_address: String,
    pub query_id: u64,
    pub unbonding_delegations: Vec<UnbondingEntry>,
    pub last_updated_height: u64,
}

#[cw_serde]
pub struct RedeemShareItem {
    pub amount: Uint128,
    pub remote_denom: String,
    pub local_denom: String,
}

pub struct UnbondingDelegationIndexes<'a> {
    pub query_id: UniqueIndex<'a, u64, UnbondingDelegation, String>,
}

impl<'a> IndexList<UnbondingDelegation> for UnbondingDelegationIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<UnbondingDelegation>> + '_> {
        let v: Vec<&dyn Index<UnbondingDelegation>> = vec![&self.query_id];
        Box::new(v.into_iter())
    }
}

pub type Recipient = str;
pub const LOCAL_DENOM: &str = "untrn";
pub const ICA_ID: &str = "DROP";

pub use reply_msg::ReplyMsg;
mod reply_msg {
    const OFFSET: u64 = u16::BITS as u64;
    const SUDO_PAYLOAD: u64 = 1 << OFFSET;
    const IBC_TRANSFER: u64 = 2 << OFFSET;
    const KV_DELEGATIONS_AND_BALANCE: u64 = 3 << OFFSET;
    const KV_NON_NATIVE_REWARDS_BALANCES: u64 = 4 << OFFSET;
    const KV_UNBONDING_DELEGATIONS_LOWER_BOUND: u64 = 5 << OFFSET;
    const KV_UNBONDING_DELEGATIONS_UPPER_BOUND: u64 =
        KV_UNBONDING_DELEGATIONS_LOWER_BOUND + u16::MAX as u64;

    #[cosmwasm_schema::cw_serde]
    pub enum ReplyMsg {
        SudoPayload,
        IbcTransfer,
        KvDelegationsAndBalance,
        KvNonNativeRewardsBalances,
        KvUnbondingDelegations { validator_index: u16 },
    }

    impl ReplyMsg {
        pub fn to_reply_id(&self) -> u64 {
            match self {
                ReplyMsg::SudoPayload => SUDO_PAYLOAD,
                ReplyMsg::IbcTransfer => IBC_TRANSFER,
                ReplyMsg::KvDelegationsAndBalance => KV_DELEGATIONS_AND_BALANCE,
                ReplyMsg::KvNonNativeRewardsBalances => KV_NON_NATIVE_REWARDS_BALANCES,
                ReplyMsg::KvUnbondingDelegations { validator_index } => {
                    KV_UNBONDING_DELEGATIONS_LOWER_BOUND | *validator_index as u64
                }
            }
        }

        pub fn from_reply_id(reply_id: u64) -> Self {
            match reply_id {
                SUDO_PAYLOAD => Self::SudoPayload,
                IBC_TRANSFER => Self::IbcTransfer,
                KV_DELEGATIONS_AND_BALANCE => Self::KvDelegationsAndBalance,
                KV_NON_NATIVE_REWARDS_BALANCES => Self::KvNonNativeRewardsBalances,
                validator_index @ KV_UNBONDING_DELEGATIONS_LOWER_BOUND
                    ..=KV_UNBONDING_DELEGATIONS_UPPER_BOUND => Self::KvUnbondingDelegations {
                    validator_index: validator_index as u16,
                },
                _ => unreachable!(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn enum_variant_from_reply_id() {
            assert_eq!(ReplyMsg::from_reply_id(SUDO_PAYLOAD), ReplyMsg::SudoPayload);
            assert_eq!(ReplyMsg::from_reply_id(IBC_TRANSFER), ReplyMsg::IbcTransfer);
            assert_eq!(
                ReplyMsg::from_reply_id(KV_DELEGATIONS_AND_BALANCE),
                ReplyMsg::KvDelegationsAndBalance
            );
        }

        #[test]
        fn enum_variant_to_reply_id() {
            assert_eq!(ReplyMsg::SudoPayload.to_reply_id(), SUDO_PAYLOAD);
            assert_eq!(ReplyMsg::IbcTransfer.to_reply_id(), IBC_TRANSFER);
            assert_eq!(
                ReplyMsg::KvDelegationsAndBalance.to_reply_id(),
                KV_DELEGATIONS_AND_BALANCE
            );
        }

        mod kv_unbonding_delegations_from_reply_id {
            use super::*;

            #[test]
            fn lower_bound() {
                assert_eq!(
                    ReplyMsg::from_reply_id(KV_UNBONDING_DELEGATIONS_LOWER_BOUND),
                    ReplyMsg::KvUnbondingDelegations { validator_index: 0 }
                );
            }

            #[test]
            fn upper_bound() {
                assert_eq!(
                    ReplyMsg::from_reply_id(KV_UNBONDING_DELEGATIONS_UPPER_BOUND),
                    ReplyMsg::KvUnbondingDelegations {
                        validator_index: u16::MAX,
                    }
                );
            }

            #[test]
            fn normal() {
                assert_eq!(
                    ReplyMsg::from_reply_id(KV_UNBONDING_DELEGATIONS_LOWER_BOUND + 2),
                    ReplyMsg::KvUnbondingDelegations { validator_index: 2 }
                );
            }
        }

        mod kv_unbonding_delegations_to_reply_id {
            use super::*;

            #[test]
            fn lower_bound() {
                assert_eq!(
                    ReplyMsg::KvUnbondingDelegations { validator_index: 0 }.to_reply_id(),
                    KV_UNBONDING_DELEGATIONS_LOWER_BOUND
                );
            }

            #[test]
            fn upper_bound() {
                assert_eq!(
                    ReplyMsg::KvUnbondingDelegations {
                        validator_index: u16::MAX
                    }
                    .to_reply_id(),
                    KV_UNBONDING_DELEGATIONS_UPPER_BOUND
                );
            }

            #[test]
            fn normal() {
                assert_eq!(
                    ReplyMsg::KvUnbondingDelegations { validator_index: 2 }.to_reply_id(),
                    KV_UNBONDING_DELEGATIONS_LOWER_BOUND + 2
                );
            }
        }

        #[test]
        #[should_panic]
        fn invalid_reply_id() {
            ReplyMsg::from_reply_id(KV_UNBONDING_DELEGATIONS_UPPER_BOUND + 1);
        }
    }
}
