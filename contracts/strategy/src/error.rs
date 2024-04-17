use cosmwasm_std::StdError;
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

    #[error("Incoming deposit and calculated deposit are not equal")]
    WrongDepositAndCalculation {},

    #[error("Incoming withdraw and calculated withdraw are not equal")]
    WrongWithdrawAndCalculation {},
}

pub type ContractResult<T> = Result<T, ContractError>;
