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

    #[error("InvalidTransaction")]
    InvalidTransaction {},

    #[error("ICA balance is zero")]
    ICABalanceZero {},

    #[error("Puppeteer response is not received")]
    PuppeteerResponseIsNotReceived {},

    #[error("Unbonded amount is not set")]
    UnbondedAmountIsNotSet {},

    #[error("Non Native rewards denom not found {denom}")]
    NonNativeRewardsDenomNotFound { denom: String },

    #[error("Puppereer balance is outdated: ICA balance height {ica_height}, puppeteer balance height {puppeteer_height}")]
    PuppeteerBalanceOutdated {
        ica_height: u64,
        puppeteer_height: u64,
    },
}

pub type ContractResult<T> = Result<T, ContractError>;
