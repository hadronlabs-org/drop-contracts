use crate::msg::token::DenomMetadata;
use crate::state::factory::{CodeIds, PreInstantiatedContracts, RemoteOpts};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Decimal, Uint128};
use cw_ownable::cw_ownable_execute;
use drop_macros::pausable;
use neutron_sdk::bindings::msg::NeutronMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub code_ids: CodeIds,
    pub pre_instantiated_contracts: PreInstantiatedContracts,
    pub remote_opts: RemoteOpts,
    pub salt: String,
    pub subdenom: String,
    pub token_metadata: DenomMetadata,
    pub base_denom: String,
    pub local_denom: String,
    pub core_params: CoreParams,
    pub fee_params: Option<FeeParams>,
}

#[cw_serde]
pub struct FeeParams {
    pub fee: Decimal, // 0 - 1
    pub fee_address: String,
}

#[cw_serde]
pub struct CoreParams {
    pub idle_min_interval: u64,
    pub unbonding_period: u64,
    pub unbonding_safe_period: u64,
    pub unbond_batch_switch_time: u64,
    pub icq_update_delay: u64, // blocks
}

#[cw_serde]
pub struct LsmShareBondParams {
    pub lsm_min_bond_amount: Uint128,
    pub lsm_redeem_threshold: u64,    //amount of lsm denoms
    pub lsm_redeem_max_interval: u64, //seconds
}

#[cw_serde]
pub enum UpdateConfigMsg {
    Core(Box<crate::state::core::ConfigOptional>),
    ValidatorsSet(crate::state::validatorset::ConfigOptional),
}

#[cw_serde]
pub enum ProxyMsg {
    ValidatorSet(ValidatorSetMsg),
}

#[cw_serde]
pub enum ValidatorSetMsg {
    UpdateValidators {
        validators: Vec<crate::msg::validatorset::ValidatorData>,
    },
}

#[cw_ownable_execute]
#[pausable]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig(Box<UpdateConfigMsg>),
    Proxy(ProxyMsg),
    AdminExecute { msgs: Vec<CosmosMsg<NeutronMsg>> },
}
#[cw_serde]
pub struct MigrateMsg {}

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(std::collections::HashMap<String, String>)]
    State {},
    #[returns(std::collections::HashMap<String, String>)]
    PauseInfo {},
}

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum OwnerQueryMsg {}
