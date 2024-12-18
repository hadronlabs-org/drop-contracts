use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as CosmosCoin;
use cosmos_sdk_proto::cosmos::staking::v1beta1::{
    Delegation, Params, Validator as CosmosValidator,
};
use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{from_json, Addr, Decimal256, StdError, Timestamp, Uint128, Uint256};
use cw_storage_plus::{Item, Map};
use drop_helpers::version::version_to_u32;
use neutron_sdk::bindings::types::StorageValue;
use neutron_sdk::{
    interchain_queries::v045::{helpers::deconstruct_account_denom_balance_key, types::Balances},
    NeutronError, NeutronResult,
};
use prost::Message;
use std::ops::Div;
use std::str::FromStr;

pub const DECIMAL_PLACES: u32 = 18;
const DECIMAL_FRACTIONAL: u128 = 10u128.pow(DECIMAL_PLACES);

pub const DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK: Map<u64, u64> =
    Map::new("delegations_and_balances_query_id_chunk");

pub const LAST_COMPLETE_DELEGATIONS_AND_BALANCES_KEY: Item<u64> =
    Item::new("last_complete_delegations_and_balances_key");

pub const DELEGATIONS_AND_BALANCES: Map<u64, BalancesAndDelegationsState<BalancesAndDelegations>> =
    Map::new("delegations_and_balances");

#[cw_serde]
pub struct BalancesAndDelegationsState<X: ResultReconstruct + std::fmt::Debug + Serialize + Clone> {
    pub data: X,
    pub remote_height: u64,
    pub local_height: u64,
    pub timestamp: Timestamp,
    pub collected_chunks: Vec<u64>,
}

pub trait ResultReconstruct {
    fn reconstruct(
        storage_values: &[StorageValue],
        version: &str,
        denom: Option<&str>,
    ) -> NeutronResult<Self>
    where
        Self: Sized;
}

#[cw_serde]
pub struct MultiBalances {
    pub coins: Vec<cosmwasm_std::Coin>,
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

impl ResultReconstruct for BalancesAndDelegations {
    fn reconstruct(
        storage_values: &[neutron_sdk::bindings::types::StorageValue],
        version: &str,
        _denom: Option<&str>,
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
            let amount: Uint128 =
                Uint128::from_str(&String::from_utf8(kv.value.to_vec()).map_err(|_| {
                    NeutronError::InvalidQueryResultFormat("Invalid utf8".to_string())
                })?)?;

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

impl ResultReconstruct for MultiBalances {
    //TODO: fix in sdk and remove this
    fn reconstruct(
        storage_values: &[StorageValue],
        version: &str,
        _: Option<&str>,
    ) -> NeutronResult<MultiBalances> {
        let mut coins: Vec<cosmwasm_std::Coin> = Vec::with_capacity(storage_values.len());
        for kv in storage_values {
            if kv.value.len() > 0 {
                let (_, denom) = deconstruct_account_denom_balance_key(kv.key.to_vec())?;
                let amount: Uint128 = match version_to_u32(version)? {
                    ver if ver >= version_to_u32("0.47.0")? => {
                        // Directly parse Uint128 from the string obtained from kv.value
                        Uint128::from_str(&String::from_utf8(kv.value.to_vec()).map_err(|_| {
                            NeutronError::InvalidQueryResultFormat("Invalid utf8".to_string())
                        })?)
                    }
                    // For versions below "0.47.0", use the existing balance.amount
                    _ => {
                        let balance: CosmosCoin = CosmosCoin::decode(kv.value.as_slice())?;
                        Uint128::from_str(balance.amount.as_str())
                    }
                }?;
                coins.push(cosmwasm_std::Coin::new(amount.u128(), denom));
            }
        }
        Ok(MultiBalances { coins })
    }
}
