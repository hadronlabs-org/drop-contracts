use cosmwasm_std::StdError;
use thiserror::Error;

#[macro_export]
macro_rules! is_paused {
    ($pause:expr, $deps:expr, $env:expr, $field:ident) => {
        match (($pause).load(($deps).storage)?).pause {
            PauseType::Switch { $field, .. } => $field,
            PauseType::Height { $field, .. } => $field <= ($env).block.height,
        }
    };
}

#[derive(Error, Debug, PartialEq)]
pub enum PauseError {
    #[error("Contract execution is paused")]
    Paused {},

    #[error("{0}")]
    Std(#[from] StdError),
}
