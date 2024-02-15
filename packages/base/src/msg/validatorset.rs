use crate::state::validatorset::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};

use crate::state::{provider_proposals::ProposalInfo, validatorset::ValidatorInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub stats_contract: String,
    pub provider_proposals_contract: String,
}

#[cw_serde]
pub struct ValidatorData {
    pub valoper_address: String,
    pub weight: u64,
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

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<Addr>,
        stats_contract: Option<Addr>,
        provider_proposals_contract: Option<Addr>,
    },
    UpdateValidators {
        validators: Vec<ValidatorData>,
    },
    UpdateValidator {
        validator: ValidatorData,
    },
    UpdateValidatorsInfo {
        validators: Vec<ValidatorInfoUpdate>,
    },
    UpdateValidatorsVoting {
        proposal: ProposalInfo,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::validatorset::Config)]
    Config {},
    #[returns(crate::state::validatorset::ValidatorInfo)]
    Validator { valoper: Addr },
    #[returns(Vec<crate::state::validatorset::ValidatorInfo>)]
    Validators {},
}

#[cw_serde]
pub struct MigrateMsg {}
