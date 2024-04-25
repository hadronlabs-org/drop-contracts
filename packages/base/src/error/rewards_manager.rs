use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use drop_helpers::pause::PauseError;
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

    #[error("unauthorized")]
    Unauthorized,

    #[error(transparent)]
    PauseError(#[from] PauseError),

    #[error("Denoms list is empty")]
    EmptyDenomsList,

    #[error("Handler for this denom already exists")]
    DenomHandlerAlreadyExists,
}

pub type ContractResult<T> = Result<T, ContractError>;
