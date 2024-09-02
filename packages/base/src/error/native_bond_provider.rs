use cosmwasm_std::{StdError, Uint128};
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

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Invalid denom")]
    InvalidDenom {},

    #[error("Message is not supported")]
    MessageIsNotSupported {},

    #[error("Puppeteer error: {message}")]
    PuppeteerError { message: String },

    #[error("Unknown reply id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Not enough coins to delegate. Min stake amount: {min_stake_amount}, non staked balance: {non_staked_balance}")]
    NotEnoughToDelegate {
        min_stake_amount: Uint128,
        non_staked_balance: Uint128,
    },

    #[error("Invalid State: {reason}")]
    InvalidState { reason: String },

    #[error("Puppeteer ICA is not registered")]
    IcaNotRegistered {},
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
