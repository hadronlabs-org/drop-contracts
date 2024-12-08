use cosmos_sdk_proto::cosmos::staking::v1beta1::{
    Delegation, Params, Validator as CosmosValidator,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::{from_json, Decimal256, StdError, Uint128, Uint256};
use cw_storage_plus::Item;

use drop_helpers::version::version_to_u32;
use drop_puppeteer_base::{
    r#trait::PuppeteerReconstruct,
    state::{BalancesAndDelegationsState, BaseConfig},
};
use neutron_sdk::{
    interchain_queries::v045::{helpers::deconstruct_account_denom_balance_key, types::Balances},
    NeutronError, NeutronResult,
};
use prost::Message;
use std::ops::Div;
use std::str::FromStr;

use crate::msg::puppeteer::MultiBalances;

#[cw_serde]
pub struct ConfigOptional {
    pub connection_id: Option<String>,
    pub port_id: Option<String>,
    pub update_period: Option<u64>,
    pub remote_denom: Option<String>,
    pub allowed_senders: Option<Vec<String>>,
    pub transfer_channel_id: Option<String>,
    pub sdk_version: Option<String>,
    pub factory_contract: Option<Addr>,
    pub timeout: Option<u64>,
}

#[cw_serde]
pub struct Config {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64, // update period in seconds for ICQ queries
    pub remote_denom: String,
    pub allowed_senders: Vec<Addr>,
    pub transfer_channel_id: String,
    pub sdk_version: String,
    pub timeout: u64, // timeout for interchain transactions in seconds
    pub delegations_queries_chunk_size: u32,
    pub factory_contract: Addr,
}

impl BaseConfig for Config {
    fn connection_id(&self) -> String {
        self.connection_id.clone()
    }

    fn update_period(&self) -> u64 {
        self.update_period
    }
}

#[cw_serde]
pub enum KVQueryType {
    UnbondingDelegations,
    DelegationsAndBalance,
    NonNativeRewardsBalances,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const NON_NATIVE_REWARD_BALANCES: Item<BalancesAndDelegationsState<MultiBalances>> =
    Item::new("non_native_reward_balances");

pub const DECIMAL_PLACES: u32 = 18;
const DECIMAL_FRACTIONAL: u128 = 10u128.pow(DECIMAL_PLACES);

#[cw_serde]
#[derive(Default)]
pub struct Pause {
    pub delegate: bool,
    pub undelegate: bool,
    pub claim_rewards_and_optionally_transfer: bool,
    pub tokenize_share: bool,
    pub redeem_shares: bool,
}
pub const PAUSE: Item<Pause> = Item::new("pause");

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

impl PuppeteerReconstruct for BalancesAndDelegations {
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
