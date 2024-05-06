use crate::state::{CodeIds, RemoteOpts};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Uint128};
use cw_ownable::cw_ownable_execute;
use drop_macros::pausable;
use drop_staking_base::msg::token::DenomMetadata;
use neutron_sdk::bindings::msg::NeutronMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub code_ids: CodeIds,
    pub remote_opts: RemoteOpts,
    pub salt: String,
    pub subdenom: String,
    pub token_metadata: DenomMetadata,
    pub sdk_version: String,
    pub base_denom: String,
    pub core_params: CoreParams,
    pub staker_params: StakerParams,
}

#[cw_serde]
pub struct CoreParams {
    pub idle_min_interval: u64,
    pub puppeteer_timeout: u64,
    pub unbonding_period: u64,
    pub unbonding_safe_period: u64,
    pub unbond_batch_switch_time: u64,
    pub lsm_min_bond_amount: Uint128,
    pub lsm_redeem_threshold: u64,
    pub lsm_redeem_max_interval: u64, //seconds
    pub bond_limit: Option<Uint128>,
    pub min_stake_amount: Uint128,
    pub icq_update_delay: u64, // blocks
}

#[cw_serde]
pub struct StakerParams {
    pub min_stake_amount: Uint128,
    pub min_ibc_transfer: Uint128,
}

#[cw_serde]
pub enum UpdateConfigMsg {
    Core(Box<drop_staking_base::state::core::ConfigOptional>),
    ValidatorsSet(drop_staking_base::state::validatorset::ConfigOptional),
}

#[cw_serde]
pub enum ProxyMsg {
    ValidatorSet(ValidatorSetMsg),
    Core(CoreMsg),
}

#[cw_serde]
pub enum CoreMsg {
    UpdateNonNativeRewardsReceivers {
        items: Vec<drop_staking_base::state::core::NonNativeRewardsItem>,
    },
    Pause {},
    Unpause {},
}

#[cw_serde]
pub enum ValidatorSetMsg {
    UpdateValidators {
        validators: Vec<drop_staking_base::msg::validatorset::ValidatorData>,
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
pub enum MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::State)]
    State {},
    #[returns(crate::state::PauseInfoResponse)]
    PauseInfo {},
}
