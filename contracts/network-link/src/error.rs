use cosmwasm_std::{OverflowError, StdError};
use cw_ownable::OwnershipError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("Invalid Funds: {reason}")]
    InvalidFunds { reason: String },

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Wrong Prefix")]
    WrongPrefix {},

    #[error("Wrong Address")]
    WrongAddress {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
