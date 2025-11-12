use cosmwasm_std::StdError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] neutron_sdk::NeutronError),

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("{0}")]
    OwnershipError(#[from] cw_ownable::OwnershipError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("nothing to mint")]
    NothingToMint,

    #[error("unknown reply id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Semver parsing error: {0}")]
    SemVer(String),

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
