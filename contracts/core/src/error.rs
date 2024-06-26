use cosmwasm_std::{OverflowError, StdError, Uint128};
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

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

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

    #[error("Invalid denom")]
    InvalidDenom {},

    #[error("Idle min interval is not reached")]
    IdleMinIntervalIsNotReached {},

    #[error("Unbonding time is too close")]
    UnbondingTimeIsClose {},

    #[error("Pump address is not set")]
    PumpAddressIsNotSet {},

    #[error("Emergency address is not set")]
    EmergencyAddressIsNotSet {},

    #[error("InvalidTransaction")]
    InvalidTransaction {},

    #[error("ICA balance is zero")]
    ICABalanceZero {},

    #[error("Bond amount is less than minimum LSM bond amount: {min_stake_amount}. Provided: {bond_amount}")]
    LSMBondAmountIsBelowMinimum {
        min_stake_amount: Uint128,
        bond_amount: Uint128,
    },

    #[error("Puppeteer response is not received")]
    PuppeteerResponseIsNotReceived {},

    #[error("Unbonded amount is not set")]
    UnbondedAmountIsNotSet {},

    #[error("Non Native rewards denom not found {denom}")]
    NonNativeRewardsDenomNotFound { denom: String },

    #[error("Puppeteer balance is outdated: ICA balance height {ica_height}, puppeteer balance height {puppeteer_height}")]
    PuppeteerBalanceOutdated {
        ica_height: u64,
        puppeteer_height: u64,
    },
    #[error("Bond limit exceeded")]
    BondLimitExceeded {},

    #[error("Previous staking was failed")]
    PreviousStakingWasFailed {},

    #[error(transparent)]
    PauseError(#[from] PauseError),
}

pub type ContractResult<T> = Result<T, ContractError>;
