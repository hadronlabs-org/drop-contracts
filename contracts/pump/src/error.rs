use cosmwasm_std::{OverflowError, StdError};
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
    OverflowError(#[from] OverflowError),

    #[error("ICA is not registered")]
    IcaNotRegistered {},

    #[error("ICA registration is in progress right now")]
    IcaInProgress {},

    #[error("ICA is already registered")]
    IcaAlreadyRegistered {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Funds: {reason}")]
    InvalidFunds { reason: String },

    #[error("Unknown sudo response")]
    UnknownResponse {},

    #[error("No destination address is set")]
    NoDestinationAddress {},

    #[error("No destination port is set")]
    NoDestinationPort {},

    #[error("No destination channel is set")]
    NoDestinationChannel {},

    #[error("Refundee is not set")]
    RefundeeIsNotSet {},

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
