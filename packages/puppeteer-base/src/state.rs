use cosmos_sdk_proto::cosmos::{
    base::v1beta1::Coin as CosmosCoin,
    staking::v1beta1::{Delegation, Params, Validator as CosmosValidator},
};
use cosmwasm_std::{from_json, Addr, Decimal256, StdError, Timestamp, Uint128, Uint256};

use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, UniqueIndex};
use drop_helpers::{ica::Ica, version::version_to_u32};
use neutron_sdk::{
    interchain_queries::v045::{
        helpers::deconstruct_account_denom_balance_key,
        types::{Balances, UnbondingEntry},
    },
    NeutronError, NeutronResult,
};
use prost::Message;
use serde::{de::DeserializeOwned, Serialize};
use std::ops::Div;
use std::str::FromStr;

use crate::{msg::Transaction, r#trait::PuppeteerReconstruct};

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
    pub kv_queries: Map<'a, u64, U>,
    pub last_complete_delegations_and_balances_key: Item<'a, u64>,
    pub delegations_and_balances:
        Map<'a, &'a u64, BalancesAndDelegationsState<BalancesAndDelegations>>,
    pub delegations_and_balances_query_id_chunk: Map<'a, u64, u16>, // Map <query_id, chunk_id>
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
            transfer_channel_id: Item::new("transfer_channel_id"),
            kv_queries: Map::new("kv_queries"),
            last_complete_delegations_and_balances_key: Item::new(
                "last_complete_delegations_and_balances_key",
            ),
            delegations_and_balances: Map::new("delegations_and_balance"),
            delegations_and_balances_query_id_chunk: Map::new(
                "delegations_and_balance_reply_id_storage",
            ),
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
pub struct BalancesAndDelegationsState<
    X: PuppeteerReconstruct + std::fmt::Debug + Serialize + Clone,
> {
    pub data: X,
    pub remote_height: u64,
    pub local_height: u64,
    pub timestamp: Timestamp,
    pub collected_chunks: Vec<u16>,
}

#[cw_serde]
pub struct BalancesAndDelegations {
    pub balances: Balances,
    pub delegations: Delegations,
}

#[cw_serde]
pub struct Delegations {
    pub delegations: Vec<DropDelegation>,
}

#[cw_serde]
pub struct DropDelegation {
    pub delegator: Addr,
    /// A validator address (e.g. cosmosvaloper1...)
    pub validator: String,
    /// How much we have locked in the delegation
    pub amount: cosmwasm_std::Coin,
    /// How many shares the delegator has in the validator
    pub share_ratio: Decimal256,
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

pub const DECIMAL_PLACES: u32 = 18;
const DECIMAL_FRACTIONAL: u128 = 10u128.pow(DECIMAL_PLACES);

impl PuppeteerReconstruct for BalancesAndDelegations {
    fn reconstruct(
        storage_values: &[neutron_sdk::bindings::types::StorageValue],
        version: &str,
    ) -> NeutronResult<Self> {
        let version = version_to_u32(version)?;
        if storage_values.is_empty() {
            return Err(NeutronError::InvalidQueryResultFormat(
                "storage_values length is 0".into(),
            ));
        }
        let mut coins: Vec<cosmwasm_std::Coin> = Vec::with_capacity(storage_values.len());
        let kv = &storage_values[0];
        if kv.value.len() > 0 {
            let (_, denom) = deconstruct_account_denom_balance_key(kv.key.to_vec())?;
            let amount: Uint128 = match version {
                ver if ver >= version_to_u32("0.47.0")? => {
                    // Directly parse Uint128 from the string obtained from kv.value
                    Uint128::from_str(&String::from_utf8(kv.value.to_vec()).map_err(|_| {
                        NeutronError::InvalidQueryResultFormat("Invalid utf8".to_string())
                    })?)
                }
                // For versions below "0.47.0", use the existing balance.amount
                _ => {
                    let balance = CosmosCoin::decode(kv.value.as_slice())?;
                    Uint128::from_str(balance.amount.as_str())
                }
            }?;
            coins.push(cosmwasm_std::Coin::new(amount.u128(), denom));
        }
        let mut delegations: Vec<DropDelegation> =
            Vec::with_capacity((storage_values.len() - 2) / 2);
        // first StorageValue is denom
        if !storage_values[1].value.is_empty() {
            let denom = match version {
                ver if ver >= version_to_u32("0.47.0")? => {
                    // Parse as Params and get bond_denom
                    Params::decode(storage_values[1].value.as_slice())?.bond_denom
                }
                // For versions below "0.47.0", parse as string
                _ => from_json(&storage_values[1].value)?,
            };
            for chunk in storage_values[2..].chunks(2) {
                if chunk[0].value.is_empty() {
                    // Incoming delegation can actually be empty, this just means that delegation
                    // is not present on remote chain, which is to be expected. So, if it doesn't
                    // exist, we can safely skip this and following chunk.
                    continue;
                }
                let delegation_sdk: Delegation = Delegation::decode(chunk[0].value.as_slice())?;

                let mut delegation_std = DropDelegation {
                    delegator: Addr::unchecked(delegation_sdk.delegator_address.as_str()),
                    validator: delegation_sdk.validator_address,
                    amount: Default::default(),
                    share_ratio: Decimal256::one(),
                };

                if chunk[1].value.is_empty() {
                    // At this point, incoming validator cannot be empty, that would be invalid,
                    // because delegation is already defined, so, building `cosmwasm_std::Delegation`
                    // from this data is impossible, incoming data is corrupted.post
                    return Err(NeutronError::InvalidQueryResultFormat(
                        "validator is empty".into(),
                    ));
                }
                let validator: CosmosValidator =
                    CosmosValidator::decode(chunk[1].value.as_slice())?;

                let delegation_shares = Decimal256::from_atomics(
                    Uint128::from_str(&delegation_sdk.shares)?,
                    DECIMAL_PLACES,
                )?;

                let delegator_shares = Decimal256::from_atomics(
                    Uint128::from_str(&validator.delegator_shares)?,
                    DECIMAL_PLACES,
                )?;

                let validator_tokens =
                    Decimal256::from_atomics(Uint128::from_str(&validator.tokens)?, 0)?;

                // https://github.com/cosmos/cosmos-sdk/blob/35ae2c4c72d4aeb33447d5a7af23ca47f786606e/x/staking/keeper/querier.go#L463
                // delegated_tokens = quotient(delegation.shares * validator.tokens / validator.total_shares);
                let delegated_tokens = Uint128::try_from(
                    delegation_shares
                        .checked_mul(validator_tokens)?
                        .div(delegator_shares)
                        .atomics()
                        / Uint256::from(DECIMAL_FRACTIONAL),
                )
                .map_err(|err| NeutronError::Std(StdError::ConversionOverflow { source: err }))?
                .u128();
                delegation_std.share_ratio = validator_tokens / delegator_shares;
                delegation_std.amount = cosmwasm_std::Coin::new(delegated_tokens, &denom);

                delegations.push(delegation_std);
            }
        }
        Ok(BalancesAndDelegations {
            delegations: Delegations { delegations },
            balances: Balances { coins },
        })
    }
}

pub type Recipient = str;
pub const LOCAL_DENOM: &str = "untrn";
pub const ICA_ID: &str = "DROP";

pub use reply_msg::ReplyMsg;
pub mod reply_msg {
    const OFFSET: u64 = u16::BITS as u64;
    pub const SUDO_PAYLOAD: u64 = 1 << OFFSET;
    pub const IBC_TRANSFER: u64 = 2 << OFFSET;
    pub const KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND: u64 = 3 << OFFSET;
    pub const KV_DELEGATIONS_AND_BALANCE_UPPER_BOUND: u64 =
        KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND + u16::MAX as u64;
    pub const KV_NON_NATIVE_REWARDS_BALANCES: u64 = 4 << OFFSET;
    const KV_UNBONDING_DELEGATIONS_LOWER_BOUND: u64 = 5 << OFFSET;
    const KV_UNBONDING_DELEGATIONS_UPPER_BOUND: u64 =
        KV_UNBONDING_DELEGATIONS_LOWER_BOUND + u16::MAX as u64;

    #[cosmwasm_schema::cw_serde]
    pub enum ReplyMsg {
        SudoPayload,
        IbcTransfer,
        KvDelegationsAndBalance { i: u16 },
        KvNonNativeRewardsBalances,
        KvUnbondingDelegations { validator_index: u16 },
    }

    impl ReplyMsg {
        pub fn to_reply_id(&self) -> u64 {
            match self {
                ReplyMsg::SudoPayload => SUDO_PAYLOAD,
                ReplyMsg::IbcTransfer => IBC_TRANSFER,
                ReplyMsg::KvDelegationsAndBalance { i } => {
                    KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND | *i as u64
                }
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
                KV_NON_NATIVE_REWARDS_BALANCES => Self::KvNonNativeRewardsBalances,
                i @ KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND
                    ..=KV_DELEGATIONS_AND_BALANCE_UPPER_BOUND => {
                    Self::KvDelegationsAndBalance { i: i as u16 }
                }
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
        }

        #[test]
        fn enum_variant_to_reply_id() {
            assert_eq!(ReplyMsg::SudoPayload.to_reply_id(), SUDO_PAYLOAD);
            assert_eq!(ReplyMsg::IbcTransfer.to_reply_id(), IBC_TRANSFER);
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

        mod kv_delegations_from_reply_id {
            use super::*;

            #[test]
            fn lower_bound() {
                assert_eq!(
                    ReplyMsg::from_reply_id(KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND),
                    ReplyMsg::KvDelegationsAndBalance { i: 0 }
                );
            }

            #[test]
            fn upper_bound() {
                assert_eq!(
                    ReplyMsg::from_reply_id(KV_DELEGATIONS_AND_BALANCE_UPPER_BOUND),
                    ReplyMsg::KvDelegationsAndBalance { i: u16::MAX }
                );
            }

            #[test]
            fn normal() {
                assert_eq!(
                    ReplyMsg::from_reply_id(KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND + 2),
                    ReplyMsg::KvDelegationsAndBalance { i: 2 }
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

        mod kv_delegations_to_reply_id {
            use super::*;

            #[test]
            fn lower_bound() {
                assert_eq!(
                    ReplyMsg::KvDelegationsAndBalance { i: 0 }.to_reply_id(),
                    KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND
                );
            }

            #[test]
            fn upper_bound() {
                assert_eq!(
                    ReplyMsg::KvDelegationsAndBalance { i: u16::MAX }.to_reply_id(),
                    KV_DELEGATIONS_AND_BALANCE_UPPER_BOUND
                );
            }

            #[test]
            fn normal() {
                assert_eq!(
                    ReplyMsg::KvDelegationsAndBalance { i: 2 }.to_reply_id(),
                    KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND + 2
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
