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

    #[error("no withdrawal asset was provided")]
    WithdrawalAssetExpected {},

    #[error("withdrawn amount is too big")]
    WithdrawnAmountTooBig {},

    #[error("amount to withdraw is zero")]
    NothingToWithdraw {},

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Bondings query limit exceeded")]
    QueryBondingsLimitExceeded {},

    #[error("Core replies with invalid data")]
    InvalidCoreReply {},
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
