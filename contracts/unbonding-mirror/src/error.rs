use cosmwasm_std::{CoinFromStrError, StdError};
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    CoinFromStrError(#[from] CoinFromStrError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("Wrong denom")]
    WrongDenom,

    #[error("Backup is not set")]
    BackupIsNotSet,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Wrong receiver address")]
    WrongReceiverAddress,

    #[error("Invalid prefix")]
    InvalidPrefix,

    #[error("No nft minted")]
    NoNFTMinted,

    #[error("No nft minted amount found from the event")]
    NoNFTMintedFound,

    #[error("No transfer")]
    NoTransferEvent,

    #[error("No transfer amount found from the event")]
    NoTransferAmountFound,

    #[error("Parsing nft error")]
    NFTParseError,

    #[error("Wrong bond state. Expected {expected:?}, got {got:?}")]
    WrongBondState { expected: String, got: String },

    #[error("Channel on the host chain wasn't found")]
    SourceChannelNotFound,

    #[error("IBC timeout out of range")]
    IbcTimeoutOutOfRange,

    #[error("Can't migrate from {storage_contract_name} to {contract_name}")]
    MigrationError {
        storage_contract_name: String,
        contract_name: String,
    },
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
