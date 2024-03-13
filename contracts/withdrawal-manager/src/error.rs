use cosmwasm_std::{OverflowError, StdError};
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),

    #[error("Invalid NFT: {reason}")]
    InvalidNFT { reason: String },

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Batch is not withdrawn yet")]
    BatchIsNotWithdrawn {},

    #[error("Missing unbonded amount in batch")]
    BatchAmountIsEmpty {},

    #[error("Slashing effect is not set")]
    BatchSlashingEffectIsEmpty {},
}

pub type ContractResult<T> = Result<T, ContractError>;
