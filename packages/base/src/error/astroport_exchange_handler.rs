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

    #[error("unauthorized")]
    Unauthorized,

    #[error("Max spread is zero")]
    ZeroMaxSpread,

    #[error("Max spread is too big. Max: 0.1")]
    MaxSpreadTooBig,

    #[error("Low balance to perform swap operation. Minimum: {min_amount}{denom}, current: {amount}{denom}")]
    LowBalance {
        min_amount: Uint128,
        amount: Uint128,
        denom: String,
    },

    #[error("Asset price query is failed {details}")]
    AssetPriceQueryFailed { details: String },
}

pub type ContractResult<T> = Result<T, ContractError>;
