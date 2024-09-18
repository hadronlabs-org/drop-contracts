use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::{Decimal256, StdError, Uint128, Uint256};
use drop_proto::proto::initia::mstaking::v1::{Delegation, Validator as InitiaValidator};
use drop_puppeteer_base::r#trait::PuppeteerReconstruct;
use neutron_sdk::{interchain_queries::v045::types::Balances, NeutronError, NeutronResult};
use prost::Message;
use std::ops::Div;
use std::str::FromStr;

#[cw_serde]
pub enum KVQueryType {
    UnbondingDelegations,
    DelegationsAndBalance,
    NonNativeRewardsBalances,
}

pub const DECIMAL_PLACES: u32 = 18;
const DECIMAL_FRACTIONAL: u128 = 10u128.pow(DECIMAL_PLACES);

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
        _version: &str,
        denom: Option<&str>,
    ) -> NeutronResult<Self> {
        let denom =
            denom.ok_or_else(|| NeutronError::InvalidQueryResultFormat("denom is empty".into()))?;
        if storage_values.is_empty() {
            return Err(NeutronError::InvalidQueryResultFormat(
                "storage_values length is 0".into(),
            ));
        }
        let mut coins: Vec<cosmwasm_std::Coin> = Vec::with_capacity(1);
        let kv = &storage_values[0];
        if kv.value.len() > 0 {
            if kv.value.len() < 40 {
                return Err(NeutronError::InvalidQueryResultFormat(
                    "balance value length is less than 40".into(),
                ));
            }
            // first 32 bytes in the value are the address
            // next 8 bytes - u64 is balance in LE
            let balance: u64 = u64::from_le_bytes(kv.value[32..40].try_into().unwrap());
            let coin = cosmwasm_std::Coin {
                denom: denom.to_string(),
                amount: balance.into(),
            };
            coins.push(coin);
        }
        let total_validators = (storage_values.len() - 1) / 2;
        let mut delegations: Vec<DropDelegation> = Vec::with_capacity(total_validators);

        if total_validators > 0 {
            println!("total_validators {}", total_validators);
            for chunk in storage_values[1..].chunks(2) {
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
                let validator: InitiaValidator =
                    InitiaValidator::decode(chunk[1].value.as_slice())?;

                let delegation_shares = Decimal256::from_atomics(
                    Uint128::from_str(
                        &delegation_sdk
                            .shares
                            .iter()
                            .find(|o| o.denom == denom)
                            .ok_or(NeutronError::InvalidQueryResultFormat(
                                "denom not found".to_string(),
                            ))?
                            .amount,
                    )?,
                    DECIMAL_PLACES,
                )?;

                let delegator_shares = Decimal256::from_atomics(
                    Uint128::from_str(
                        &validator
                            .delegator_shares
                            .iter()
                            .find(|o| o.denom == denom)
                            .ok_or(NeutronError::InvalidQueryResultFormat(
                                "denom not found".to_string(),
                            ))?
                            .amount,
                    )?,
                    DECIMAL_PLACES,
                )?;

                let validator_tokens = Decimal256::from_atomics(
                    Uint128::from_str(
                        &validator
                            .tokens
                            .iter()
                            .find(|o| o.denom == denom)
                            .ok_or(NeutronError::InvalidQueryResultFormat(
                                "denom not found".to_string(),
                            ))?
                            .amount,
                    )?,
                    0,
                )?;

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
                delegation_std.amount = cosmwasm_std::Coin::new(delegated_tokens, denom);

                delegations.push(delegation_std);
            }
        }
        Ok(BalancesAndDelegations {
            delegations: Delegations { delegations },
            balances: Balances { coins },
        })
    }
}
