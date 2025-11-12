use cosmwasm_std::{OverflowError, StdError};
use cw_ownable::OwnershipError;
use drop_helpers::pause::PauseError;
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),

    #[error("Invalid NFT: {reason}")]
    InvalidNFT { reason: String },

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Batch is not withdrawn yet")]
    BatchIsNotWithdrawn {},

    #[error("Missing unbonded amount in batch")]
    BatchAmountIsEmpty {},

    #[error("Slashing effect is not set")]
    BatchSlashingEffectIsEmpty {},

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error(transparent)]
    PauseError(#[from] PauseError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Can't migrate from {storage_contract_name} to {contract_name}")]
    MigrationError {
        storage_contract_name: String,
        contract_name: String,
    },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
