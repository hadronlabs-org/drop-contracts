use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_ownable::OwnershipError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),
    #[error("Could not calculcate instantiate2 address: {0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Unimplemented")]
    Unimplemented {},
    #[error("Unknown")]
    Unknown {},
}

pub type ContractResult<T> = Result<T, ContractError>;
