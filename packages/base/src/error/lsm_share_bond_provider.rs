use cosmwasm_std::{ConversionOverflowError, Decimal256RangeExceeded, OverflowError, StdError};
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
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Invalid denom")]
    InvalidDenom {},

    #[error("Message is not supported")]
    MessageIsNotSupported {},

    #[error("No delegations")]
    NoDelegations {},

    #[error("Validator info not found: {validator}")]
    ValidatorInfoNotFound { validator: String },

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("Puppeteer ICA is not registered")]
    IcaNotRegistered {},

    #[error("Invalid State: {reason}")]
    InvalidState { reason: String },

    #[error("Puppeteer error: {message}")]
    PuppeteerError { message: String },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
