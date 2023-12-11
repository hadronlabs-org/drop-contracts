use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::state::ValidatorInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub stats_contract: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<Addr>,
        stats_contract: Option<Addr>,
    },
    UpdateValidators {
        validators: Vec<ValidatorInfo>,
    },
    UpdateValidator {
        validator: ValidatorInfo,
    },
}

#[cw_serde]
pub struct MigrateMsg {}
