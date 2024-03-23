use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;
use thiserror::Error;

const PAUSED: Item<bool> = Item::new("paused");

/// Set contract on pause.
pub fn set_pause(storage: &mut dyn Storage) -> StdResult<bool> {
    PAUSED.save(storage, &true)?;
    Ok(true)
}

/// Unpause the contract.
pub fn unpause(storage: &mut dyn Storage) -> StdResult<bool> {
    PAUSED.remove(storage);
    Ok(true)
}

/// Check if the contract is paused.
pub fn is_paused(storage: &dyn Storage) -> bool {
    PAUSED
        .may_load(storage)
        .unwrap_or_default()
        .unwrap_or_default()
}

/// Assert that an account is the contract's current owner.
pub fn assert_paused(store: &dyn Storage) -> Result<(), PauseError> {
    // the sender must be the current owner
    if is_paused(store) {
        return Err(PauseError::Paused {});
    }

    Ok(())
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PauseError {
    #[error("Contract execution is paused")]
    Paused {},
}

/// Information about if the contract is currently paused.
#[cw_serde]
pub enum PauseInfoResponse {
    Paused {},
    Unpaused {},
}
