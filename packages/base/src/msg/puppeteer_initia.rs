use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use drop_helpers::version::version_to_u32;
use prost::Message;

use crate::state::puppeteer::ConfigOptional;
use crate::state::puppeteer_initia::Delegations;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as CosmosCoin;
use drop_puppeteer_base::{
    msg::{ExecuteMsg as BaseExecuteMsg, IBCTransferReason, TransferReadyBatchesMsg},
    r#trait::PuppeteerReconstruct,
    state::RedeemShareItem,
};
use neutron_sdk::{
    bindings::types::StorageValue,
    interchain_queries::v045::{helpers::deconstruct_account_denom_balance_key, types::Balances},
};
use neutron_sdk::{NeutronError, NeutronResult};
use std::str::FromStr;

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub update_period: u64,
    pub remote_denom: String,
    pub owner: Option<String>,
    pub allowed_senders: Vec<String>,
    pub transfer_channel_id: String,
    pub sdk_version: String,
    pub timeout: u64,
    pub delegations_queries_chunk_size: Option<u32>,
}

#[cw_ownable_execute]
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
    SetupProtocol {
        delegate_grantee: String,
        rewards_withdraw_address: String,
    },
    Undelegate {
        items: Vec<(String, Uint128)>,
        batch_id: u128,
        reply_to: String,
    },
    Redelegate {
        validator_from: String,
        validator_to: String,
        amount: Uint128,
        reply_to: String,
    },
    TokenizeShare {
        validator: String,
        amount: Uint128,
        reply_to: String,
    },
    RedeemShares {
        items: Vec<RedeemShareItem>,
        reply_to: String,
    },
    IBCTransfer {
        reason: IBCTransferReason,
        reply_to: String,
    },
    Transfer {
        items: Vec<(String, cosmwasm_std::Coin)>,
        reply_to: String,
    },
    ClaimRewardsAndOptionalyTransfer {
        validators: Vec<String>,
        transfer: Option<TransferReadyBatchesMsg>,
        reply_to: String,
    },
    UpdateConfig {
        new_config: ConfigOptional,
    },
}

impl ExecuteMsg {
    pub fn to_base_enum(&self) -> BaseExecuteMsg {
        match self {
            ExecuteMsg::RegisterICA {} => BaseExecuteMsg::RegisterICA {},
            ExecuteMsg::RegisterQuery {} => BaseExecuteMsg::RegisterQuery {},
            _ => unimplemented!(),
        }
    }
}

#[cw_serde]
pub struct MigrateMsg {}

pub type RemoteHeight = u64;
pub type LocalHeight = u64;

#[cw_serde]
pub struct DelegationsResponse {
    pub delegations: Delegations,
    pub remote_height: u64,
    pub local_height: u64,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct BalancesResponse {
    pub balances: Balances,
    pub remote_height: u64,
    pub local_height: u64,
    pub timestamp: Timestamp,
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryExtMsg {
    #[returns(DelegationsResponse)]
    Delegations {},
    #[returns(BalancesResponse)]
    Balances {},
    #[returns(BalancesResponse)]
    NonNativeRewardsBalances {},
    #[returns(Vec<drop_puppeteer_base::state::UnbondingDelegation>)]
    UnbondingDelegations {},
}

#[cw_serde]
pub struct MultiBalances {
    pub coins: Vec<cosmwasm_std::Coin>,
}

impl PuppeteerReconstruct for MultiBalances {
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
