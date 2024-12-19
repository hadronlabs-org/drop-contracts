use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid Ica")]
    InvalidIca,
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
