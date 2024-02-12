use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal};

use crate::state::validatorset::{ConfigOptional, ValidatorInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub stats_contract: String,
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
        new_config: ConfigOptional,
    },
    UpdateValidators {
        validators: Vec<ValidatorData>,
    },
    UpdateValidator {
        validator: ValidatorData,
    },
    UpdateValidatorInfo {
        validators: Vec<ValidatorInfoUpdate>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::validatorset::Config)]
    Config {},
    #[returns(ValidatorInfo)]
    Validator { valoper: Addr },
    #[returns(Vec<ValidatorInfo>)]
    Validators {},
}

#[cw_serde]
pub struct MigrateMsg {}
