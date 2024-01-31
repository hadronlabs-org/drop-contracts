use cosmwasm_std::{OverflowError, StdError};
use neutron_sdk::NeutronError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),

    #[error("Invalid Funds: {reason}")]
    InvalidFunds { reason: String },

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},
}

pub type ContractResult<T> = Result<T, ContractError>;
