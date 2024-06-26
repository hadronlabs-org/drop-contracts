use crate::state::{CodeIds, RemoteOpts};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use cw_ownable::cw_ownable_execute;
use drop_macros::pausable;
use drop_staking_base::msg::token::DenomMetadata;

#[cw_serde]
pub struct InstantiateMsg {
    pub code_ids: CodeIds,
    pub remote_opts: RemoteOpts,
    pub salt: String,
    pub subdenom: String,
    pub token_metadata: DenomMetadata,
    pub sdk_version: String,
}

#[cw_serde]
pub enum CallbackMsg {
    PostInit {},
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
    pub channel: String,
    pub bond_limit: Option<Uint128>,
    pub min_stake_amount: Uint128,
}

#[cw_serde]
pub struct FeesMsg {
    pub recv_fee: Uint128,
    pub ack_fee: Uint128,
    pub timeout_fee: Uint128,
    pub register_fee: Uint128,
}

#[cw_serde]
pub enum UpdateConfigMsg {
    Core(Box<drop_staking_base::state::core::ConfigOptional>),
    ValidatorsSet(drop_staking_base::state::validatorset::ConfigOptional),
    PuppeteerFees(FeesMsg),
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
    UpdateValidator {
        validator: drop_staking_base::msg::validatorset::ValidatorData,
    },
}

#[cw_ownable_execute]
#[pausable]
#[cw_serde]
pub enum ExecuteMsg {
    Init {
        base_denom: String,
        core_params: CoreParams,
    },
    Callback(CallbackMsg),
    UpdateConfig(Box<UpdateConfigMsg>),
    Proxy(ProxyMsg),
    AdminExecute {
        addr: String,
        msg: Binary,
    },
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
