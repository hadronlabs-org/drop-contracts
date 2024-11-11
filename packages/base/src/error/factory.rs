use cosmwasm_std::{CheckedFromRatioError, Instantiate2AddressError, OverflowError, StdError};
use cw_ownable::OwnershipError;
use neutron_sdk::NeutronError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    NeutronError(#[from] NeutronError),
    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),
    #[error("{0}")]
    CheckedFromRatioError(#[from] CheckedFromRatioError),
    #[error("{0}")]
    OverflowError(#[from] OverflowError),
    #[error("Could not calculcate instantiate2 address: {0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Unimplemented")]
    Unimplemented {},
    #[error("Unknown")]
    Unknown {},
    #[error("Semver parsing error: {0}")]
    SemVer(String),
    #[error("Contract address not found: {name}")]
    ContractAddressNotFound { name: String },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
