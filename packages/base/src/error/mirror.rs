use cosmwasm_std::{CoinFromStrError, StdError};
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    CoinFromStrError(#[from] CoinFromStrError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("Wrong denom")]
    WrongDenom,

    #[error("Backup is not set")]
    BackupIsNotSet,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Wrong receiver address")]
    WrongReceiverAddress,

    #[error("No tokens minted")]
    NoTokensMinted,

    #[error("Invalid prefix")]
    InvalidPrefix,

    #[error("No tokens minted amount found")]
    NoTokensMintedAmountFound,

    #[error("Wrong bond state. Expected {expected:?}, got {got:?}")]
    WrongBondState { expected: String, got: String },

    #[error("Channel on the host chain wasn't found")]
    SourceChannelNotFound,
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
