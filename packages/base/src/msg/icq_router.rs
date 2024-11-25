use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal256, Timestamp};
use neutron_sdk::interchain_queries::v047::types::Balances;

use crate::state::icq_router::ConfigOptional;

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::icq_router::Config)]
    Config {},
    #[returns(DelegationsData)]
    Delegations {},
    #[returns(BalancesData)]
    Balances {},
    #[returns(BalancesData)]
    NonNativeRewardsBalances {},
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateValidators { validators: Vec<String> },
    UpdateConfig { new_config: ConfigOptional },
    UpdateBalances { balances: BalancesData },
    UpdateDelegations { delegations: DelegationsData },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub adapter: String,
    pub owner: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {}

pub type RemoteHeight = u64;
pub type LocalHeight = u64;

#[cw_serde]
pub struct DelegationsData {
    pub delegations: Delegations,
    pub remote_height: u64,
    pub local_height: u64,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct BalancesData {
    pub balances: Balances,
    pub remote_height: u64,
    pub local_height: u64,
    pub timestamp: Timestamp,
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
