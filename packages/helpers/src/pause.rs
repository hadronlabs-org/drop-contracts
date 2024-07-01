use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::Item;
use thiserror::Error;

const PAUSED: Item<bool> = Item::new("paused");

/// Set contract on pause.
pub fn set_pause(storage: &mut dyn Storage) -> StdResult<()> {
    PAUSED.save(storage, &true)?;
    Ok(())
}

/// Unpause the contract.
pub fn unpause(storage: &mut dyn Storage) {
    PAUSED.remove(storage)
}

/// Return paused/unpaused state.
pub fn is_paused(storage: &dyn Storage) -> StdResult<bool> {
    Ok(PAUSED.may_load(storage)?.unwrap_or(false))
}

/// Check that contract is not paused. If it is, return error.
pub fn pause_guard(store: &dyn Storage) -> Result<(), PauseError> {
    if is_paused(store)? {
        return Err(PauseError::Paused {});
    }

    Ok(())
}

#[derive(Error, Debug, PartialEq)]
pub enum PauseError {
    #[error("Contract execution is paused")]
    Paused {},

    #[error("{0}")]
    Std(#[from] StdError),
}

/// Information about if the contract is currently paused.
#[cw_serde]
pub enum PauseInfoResponse {
    Paused {},
    Unpaused {},
}
