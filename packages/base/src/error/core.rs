use cosmwasm_std::{
    ConversionOverflowError, Decimal256RangeExceeded, DivideByZeroError, OverflowError, StdError,
    Uint128,
};
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

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("{0}")]
    Decimal256RangeExceeded(#[from] Decimal256RangeExceeded),

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

    #[error("No delegations")]
    NoDelegations {},

    #[error("Idle min interval is not reached")]
    IdleMinIntervalIsNotReached {},

    #[error("Unbonding time is too close")]
    UnbondingTimeIsClose {},

    #[error("Pump ICA address is not set")]
    PumpIcaAddressIsNotSet {},

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

    #[error("Staker response is not received")]
    StakerResponseIsNotReceived {},

    #[error("Unbonded amount is not set")]
    UnbondedAmountIsNotSet {},

    #[error("Non Native rewards denom not found {denom}")]
    NonNativeRewardsDenomNotFound { denom: String },

    #[error(
        "Puppeteer balance is outdated: ICA height {ica_height}, control height {control_height}"
    )]
    PuppeteerBalanceOutdated {
        ica_height: u64,
        control_height: u64,
    },

    #[error("Puppeteer delegations is outdated: ICA height {ica_height}, control height {control_height}")]
    PuppeteerDelegationsOutdated {
        ica_height: u64,
        control_height: u64,
    },

    #[error("Bond limit exceeded")]
    BondLimitExceeded {},

    #[error("Unbond batches query limit exceeded")]
    QueryUnbondBatchesLimitExceeded {},

    #[error("Previous staking was failed")]
    PreviousStakingWasFailed {},

    #[error(transparent)]
    PauseError(#[from] PauseError),

    #[error("Unbonded amount must not be zero")]
    UnbondedAmountZero {},

    #[error("Requested batch is not in Withdrawn state")]
    BatchNotWithdrawn {},

    #[error("Requested batch is not in WithdrawnEmergency state")]
    BatchNotWithdrawnEmergency {},

    #[error("Unbonded amount must be less or equal to expected amount")]
    UnbondedAmountTooHigh {},

    #[error("Validator info not found: {validator}")]
    ValidatorInfoNotFound { validator: String },

    #[error("Fee must be in range [0.0, 1.0]")]
    InvalidFee {},

    #[error("Bond provider already exists")]
    BondProviderAlreadyExists {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
