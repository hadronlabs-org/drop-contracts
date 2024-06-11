use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),
}

pub type ContractResult<T> = Result<T, ContractError>;
