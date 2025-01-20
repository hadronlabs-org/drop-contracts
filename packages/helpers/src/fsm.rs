use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::Item;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Transition<T> {
    pub from: T,
    pub to: T,
}

pub struct Fsm<T: 'static> {
    pub state: Item<T>,
    pub transitions: &'static [Transition<T>],
}

impl<T: Serialize + DeserializeOwned + PartialEq> Fsm<T> {
    pub const fn new(storage_key: &'static str, transitions: &'static [Transition<T>]) -> Self {
        Self {
            state: Item::new(storage_key),
            transitions,
        }
    }

    pub fn get_current_state(&self, store: &dyn Storage) -> StdResult<T> {
        self.state
            .load(store)
            .map_err(|_| StdError::generic_err("Current FSM state not found"))
    }

    pub fn set_initial_state(&self, store: &mut dyn Storage, initial_state: T) -> StdResult<()> {
        self.state.save(store, &initial_state)
    }

    pub fn go_to(&self, store: &mut dyn Storage, to: T) -> StdResult<()> {
        let current_state = self.get_current_state(store)?;
        if self
            .transitions
            .iter()
            .any(|transition| transition.from == current_state && transition.to == to)
        {
            self.state.save(store, &to)
        } else {
            Err(StdError::generic_err("This FSM transition is not allowed"))
        }
    }
}
