use crate::state::{
    provider_proposals::ProposalInfo,
    validatorset::{ConfigOptional, ValidatorInfo},
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub stats_contract: String,
}

#[cw_serde]
pub struct ValidatorData {
    pub valoper_address: String,
    pub weight: u64,
    pub on_top: Uint128,
}

#[cw_serde]
pub struct ValidatorInfoUpdate {
    pub valoper_address: String,
    pub last_processed_remote_height: Option<u64>,
    pub last_processed_local_height: Option<u64>,
    pub last_validated_height: Option<u64>,
    pub last_commission_in_range: Option<u64>,
    pub uptime: Decimal,
    pub tombstone: bool,
    pub jailed_number: Option<u64>,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        new_config: ConfigOptional,
    },
    UpdateValidators {
        validators: Vec<ValidatorData>,
    },
    UpdateValidatorsInfo {
        validators: Vec<ValidatorInfoUpdate>,
    },
    UpdateValidatorsVoting {
        proposal: ProposalInfo,
    },
    EditOnTop {
        operations: Vec<OnTopEditOperation>,
    },
}

#[cw_serde]
pub enum OnTopEditOperation {
    Add {
        validator_address: String,
        amount: Uint128,
    },
    Subtract {
        validator_address: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub struct ValidatorResponse {
    pub validator: Option<ValidatorInfo>,
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::validatorset::Config)]
    Config {},
    #[returns(ValidatorResponse)]
    Validator { valoper: String },
    #[returns(Vec<crate::state::validatorset::ValidatorInfo>)]
    Validators {},
}

#[cw_serde]
pub struct MigrateMsg {}
