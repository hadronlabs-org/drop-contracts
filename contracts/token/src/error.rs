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
}

pub type ContractResult<T> = Result<T, ContractError>;
