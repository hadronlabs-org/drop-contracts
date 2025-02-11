use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

use crate::state::{puppeteer::Delegations, puppeteer_native::ConfigOptional};
use drop_puppeteer_base::msg::TransferReadyBatchesMsg;
use neutron_sdk::interchain_queries::v045::types::Balances;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub allowed_senders: Vec<String>,
    pub distribution_module_contract: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    SetupProtocol {
        rewards_withdraw_address: String,
    },
    Delegate {
        items: Vec<(String, Uint128)>,
        reply_to: String,
    },
    Undelegate {
        items: Vec<(String, Uint128)>,
        batch_id: u128,
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
    RegisterBalanceAndDelegatorDelegationsQuery {
        validators: Vec<String>,
    },
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryExtMsg {
    #[returns(DelegationsResponse)]
    Delegations {},
    #[returns(BalancesResponse)]
    Balances {},
    // #[returns(BalancesResponse)]
    // NonNativeRewardsBalances {},
    #[returns(Vec<drop_puppeteer_base::state::UnbondingDelegation>)]
    UnbondingDelegations {},
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::puppeteer_native::Config)]
    Config {},
    #[returns(Vec<drop_puppeteer_base::peripheral_hook::Transaction>)]
    Transactions {},
    #[returns(cosmwasm_std::Binary)]
    Extension { msg: QueryExtMsg },
}
