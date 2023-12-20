use cosmwasm_std::{DecimalRangeExceeded, OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    DecimalRangeExceeded(#[from] DecimalRangeExceeded),

    #[error("Withdraw amount is bigger than deposit amount")]
    TooBigWithdraw {},
}

pub type ContractResult<T> = Result<T, ContractError>;
