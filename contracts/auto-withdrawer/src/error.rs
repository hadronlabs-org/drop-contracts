#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] cosmwasm_std::StdError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("no ldTOKENs were provided")]
    LdTokenExpected {},

    #[error("no deposit was provided")]
    DepositExpected {},
}

pub type ContractResult<T> = Result<T, ContractError>;
