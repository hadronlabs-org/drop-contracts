use cosmwasm_std::{OverflowError, StdError};
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use neutron_sdk::NeutronError;
use prost::DecodeError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),
    #[error("{0}")]
    DecodeError(#[from] DecodeError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

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

    #[error("Invalid State: {reason}")]
    InvalidState { reason: String },

    #[error("Unknown sudo response")]
    UnknownResponse {},

    #[error("No destination address is set")]
    NoDestinationAddress {},

    #[error("No destination port is set")]
    NoDestinationPort {},

    #[error("No destination channel is set")]
    NoDestinationChannel {},
}

pub type ContractResult<T> = Result<T, ContractError>;
