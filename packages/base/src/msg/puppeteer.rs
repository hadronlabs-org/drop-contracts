use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{from_json, Addr, Decimal, Timestamp, Uint128};
use std::ops::Div;
use std::str::FromStr;

use cosmos_sdk_proto::cosmos::{
    base::v1beta1::Coin as CosmosCoin,
    staking::v1beta1::{Delegation, Validator as CosmosValidator},
};
use drop_puppeteer_base::{
    msg::{ExecuteMsg as BaseExecuteMsg, TransferReadyBatchMsg},
    state::RedeemShareItem,
};
use neutron_sdk::{
    bindings::types::StorageValue,
    interchain_queries::v045::types::{Balances, Delegations},
};
use neutron_sdk::{interchain_queries::types::KVReconstruct, NeutronError, NeutronResult};
use prost::Message;

pub const DECIMAL_PLACES: u32 = 18;
const DECIMAL_FRACTIONAL: u128 = 10u128.pow(DECIMAL_PLACES);
#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: String,
    pub allowed_senders: Vec<String>,
    pub transfer_channel_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    RegisterQuery {},
    RegisterBalanceAndDelegatorDelegationsQuery {
        validators: Vec<String>,
    },
    RegisterDelegatorUnbondingDelegationsQuery {
        validators: Vec<String>,
    },
    RegisterNonNativeRewardsBalancesQuery {
        denoms: Vec<String>,
    },
    SetFees {
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
        register_fee: Uint128,
    },
    Delegate {
        items: Vec<(String, Uint128)>,
        timeout: Option<u64>,
        reply_to: String,
    },
    Undelegate {
        items: Vec<(String, Uint128)>,
        batch_id: u128,
        timeout: Option<u64>,
        reply_to: String,
    },
    Redelegate {
        validator_from: String,
        validator_to: String,
        amount: Uint128,
        timeout: Option<u64>,
        reply_to: String,
    },
    TokenizeShare {
        validator: String,
        amount: Uint128,
        timeout: Option<u64>,
        reply_to: String,
    },
    RedeemShares {
        items: Vec<RedeemShareItem>,
        timeout: Option<u64>,
        reply_to: String,
    },
    IBCTransfer {
        timeout: u64,
        reply_to: String,
    },
    Transfer {
        items: Vec<(String, cosmwasm_std::Coin)>,
        timeout: Option<u64>,
        reply_to: String,
    },
    ClaimRewardsAndOptionalyTransfer {
        validators: Vec<String>,
        transfer: Option<TransferReadyBatchMsg>,
        timeout: Option<u64>,
        reply_to: String,
    },
}

impl ExecuteMsg {
    pub fn to_base_enum(&self) -> BaseExecuteMsg {
        match self {
            ExecuteMsg::RegisterICA {} => BaseExecuteMsg::RegisterICA {},
            ExecuteMsg::RegisterQuery {} => BaseExecuteMsg::RegisterQuery {},
            ExecuteMsg::SetFees {
                recv_fee,
                ack_fee,
                timeout_fee,
                register_fee,
            } => BaseExecuteMsg::SetFees {
                recv_fee: *recv_fee,
                ack_fee: *ack_fee,
                timeout_fee: *timeout_fee,
                register_fee: *register_fee,
            },
            _ => unimplemented!(),
        }
    }
}

#[cw_serde]
pub struct MigrateMsg {}

pub type Height = u64;

pub type DelegationsResponse = (Delegations, Height, Timestamp);
pub type BalancesResponse = (Balances, Height, Timestamp);

#[cw_serde]
pub struct FeesResponse {
    pub recv_fee: Vec<cosmwasm_std::Coin>,
    pub ack_fee: Vec<cosmwasm_std::Coin>,
    pub timeout_fee: Vec<cosmwasm_std::Coin>,
    pub register_fee: cosmwasm_std::Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryExtMsg {
    #[returns(DelegationsResponse)]
    Delegations {},
    #[returns(BalancesResponse)]
    Balances {},
    #[returns(BalancesResponse)]
    NonNativeRewardsBalances {},
    #[returns(FeesResponse)]
    Fees {},
    #[returns(Vec<drop_puppeteer_base::state::UnbondingDelegation>)]
    UnbondingDelegations {},
}

#[cw_serde]
pub struct BalancesAndDelegations {
    pub balances: Balances,
    pub delegations: Delegations,
}

impl KVReconstruct for BalancesAndDelegations {
    fn reconstruct(
        storage_values: &[neutron_sdk::bindings::types::StorageValue],
    ) -> NeutronResult<Self> {
        if storage_values.is_empty() {
            return Err(NeutronError::InvalidQueryResultFormat(
                "storage_values length is 0".into(),
            ));
        }
        let mut coins: Vec<cosmwasm_std::Coin> = Vec::with_capacity(storage_values.len());
        let kv = &storage_values[0];
        if kv.value.len() > 0 {
            let balance = CosmosCoin::decode(kv.value.as_slice())?;
            let amount = Uint128::from_str(balance.amount.as_str())?;
            coins.push(cosmwasm_std::Coin::new(amount.u128(), balance.denom));
        }
        let mut delegations: Vec<cosmwasm_std::Delegation> =
            Vec::with_capacity((storage_values.len() - 2) / 2);

        // first StorageValue is denom
        if storage_values[1].value.is_empty() {
            // Incoming denom cannot be empty, it should always be configured on chain.
            // If we receive empty denom, that means incoming data structure is corrupted
            // and we cannot build `cosmwasm_std::Delegation`'s using this data.
            return Err(NeutronError::InvalidQueryResultFormat(
                "denom is empty".into(),
            ));
        }
        let denom: String = from_json(&storage_values[1].value)?;
        // the rest are delegations and validators alternately
        for chunk in storage_values[2..].chunks(2) {
            if chunk[0].value.is_empty() {
                // Incoming delegation can actually be empty, this just means that delegation
                // is not present on remote chain, which is to be expected. So, if it doesn't
                // exist, we can safely skip this and following chunk.
                continue;
            }
            let delegation_sdk: Delegation = Delegation::decode(chunk[0].value.as_slice())?;

            let mut delegation_std = cosmwasm_std::Delegation {
                delegator: Addr::unchecked(delegation_sdk.delegator_address.as_str()),
                validator: delegation_sdk.validator_address,
                amount: Default::default(),
            };

            if chunk[1].value.is_empty() {
                // At this point, incoming validator cannot be empty, that would be invalid,
                // because delegation is already defined, so, building `cosmwasm_std::Delegation`
                // from this data is impossible, incoming data is corrupted.post
                return Err(NeutronError::InvalidQueryResultFormat(
                    "validator is empty".into(),
                ));
            }
            let validator: CosmosValidator = CosmosValidator::decode(chunk[1].value.as_slice())?;

            let delegation_shares =
                Decimal::from_atomics(Uint128::from_str(&delegation_sdk.shares)?, DECIMAL_PLACES)?;

            let delegator_shares = Decimal::from_atomics(
                Uint128::from_str(&validator.delegator_shares)?,
                DECIMAL_PLACES,
            )?;

            let validator_tokens = Decimal::from_atomics(Uint128::from_str(&validator.tokens)?, 0)?;

            // https://github.com/cosmos/cosmos-sdk/blob/35ae2c4c72d4aeb33447d5a7af23ca47f786606e/x/staking/keeper/querier.go#L463
            // delegated_tokens = quotient(delegation.shares * validator.tokens / validator.total_shares);
            let delegated_tokens = delegation_shares
                .checked_mul(validator_tokens)?
                .div(delegator_shares)
                .atomics()
                .u128()
                .div(DECIMAL_FRACTIONAL);

            delegation_std.amount = cosmwasm_std::Coin::new(delegated_tokens, &denom);

            delegations.push(delegation_std);
        }
        Ok(BalancesAndDelegations {
            delegations: Delegations { delegations },
            balances: Balances { coins },
        })
    }
}

#[cw_serde]
pub struct MultiBalances {
    pub coins: Vec<cosmwasm_std::Coin>,
}

impl KVReconstruct for MultiBalances {
    //TODO: fix in sdk and remove this
    fn reconstruct(storage_values: &[StorageValue]) -> NeutronResult<MultiBalances> {
        let mut coins: Vec<cosmwasm_std::Coin> = Vec::with_capacity(storage_values.len());
        for kv in storage_values {
            if kv.value.len() > 0 {
                let balance: CosmosCoin = CosmosCoin::decode(kv.value.as_slice())?;
                let amount = Uint128::from_str(balance.amount.as_str())?;
                coins.push(cosmwasm_std::Coin::new(amount.u128(), balance.denom));
            }
        }
        Ok(MultiBalances { coins })
    }
}
