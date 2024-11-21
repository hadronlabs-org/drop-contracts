use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),
    #[error("Semver parsing error: {0}")]
    SemVer(String),
    #[error("{0}")]
    PaymentError(#[from] PaymentError),
    #[error("Could not calculcate instantiate2 address: {0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),
    #[error("Unknown reply id {id}")]
    UnknownReplyId { id: u64 },
    #[error("{0}")]
    NeutronError(#[from] NeutronError),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
