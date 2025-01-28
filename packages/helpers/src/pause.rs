use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use thiserror::Error;

#[macro_export]
macro_rules! is_paused {
    ($pause:expr, $deps:expr, $env:expr, $field:ident) => {{
        let pause = ($pause).load(($deps).storage)?;
        let height = ($env).block.height;
        (pause.$field.from > 0 && pause.$field.to > 0)
            && (pause.$field.from <= height && height <= pause.$field.to)
    }};
}

#[derive(Error, Debug, PartialEq)]
pub enum PauseError {
    #[error("Contract execution is paused")]
    Paused {},

    #[error("{0}")]
    Std(#[from] StdError),
}

#[cw_serde]
#[derive(Default)]
pub struct Interval {
    pub from: u64,
    pub to: u64,
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.from, self.to)
    }
}
