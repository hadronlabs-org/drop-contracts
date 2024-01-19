use cosmwasm_std::{OverflowError, StdError};
use lido_helpers::fsm::FsmError;
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),

    #[error("{0}")]
    FsmError(#[from] FsmError),

    #[error("Invalid Funds: {reason}")]
    InvalidFunds { reason: String },

    #[error("Invalid NFT: {reason}")]
    InvalidNFT { reason: String },

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Batch is not unbonded yet")]
    BatchIsNotUnbonded {},

    #[error("Missing unbonded amount in batch")]
    BatchAmountIsEmpty {},

    #[error("Slashing effect is not set")]
    BatchSlashingEffectIsEmpty {},

    #[error("LD denom is not set")]
    LDDenomIsNotSet {},

    #[error("Idle min interval is not reached")]
    IdleMinIntervalIsNotReached {},

    #[error("Unbonding time is too close")]
    UnbondingTimeIsClose {},

    #[error("Pump address is not set")]
    PumpAddressIsNotSet {},

    #[error("Response is empty")]
    ResponseIsEmpty {},

    #[error("InvalidTransaction")]
    InvalidTransaction {},
}

pub type ContractResult<T> = Result<T, ContractError>;
