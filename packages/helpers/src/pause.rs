use cosmwasm_std::StdError;
use thiserror::Error;

#[macro_export]
macro_rules! is_paused {
    ($pause:expr, $deps:expr, $env:expr, $field:ident) => {{
        let pause = ($pause).load(($deps).storage)?;
        let height = ($env).block.height;
        println!("h: {:?}", height);
        println!("p: {:?}", pause.$field);
        println!("{:?}", height > 0 && pause.$field <= height);
        pause.$field > 0 && pause.$field <= height
    }};
}

#[derive(Error, Debug, PartialEq)]
pub enum PauseError {
    #[error("Contract execution is paused")]
    Paused {},

    #[error("{0}")]
    Std(#[from] StdError),
}
