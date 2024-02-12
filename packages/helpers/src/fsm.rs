use cosmwasm_schema::cw_serde;
use thiserror::Error;

#[cw_serde]
pub struct Transition<T> {
    pub from: T,
    pub to: T,
}

#[cw_serde]
pub struct Fsm<T> {
    pub initial_state: T,
    pub current_state: T,
    pub transitions: Vec<Transition<T>>,
}

#[derive(Error, Debug, PartialEq)]
pub enum FsmError {
    #[error("Current state not found")]
    CurrentStateNotFound,
    #[error("Transition is not allowed")]
    TransitionNotAllowed,
}

impl<T: PartialEq + Clone> Fsm<T> {
    pub fn new(initial_state: T, transitions: Vec<Transition<T>>) -> Self {
        Self {
            initial_state: initial_state.clone(),
            current_state: initial_state,
            transitions,
        }
    }

    pub fn go_to(&mut self, to: T) -> Result<(), FsmError> {
        let transition = self
            .transitions
            .iter()
            .find(|transition| transition.from == self.current_state && transition.to == to)
            .ok_or(FsmError::TransitionNotAllowed)?;
        self.current_state = transition.to.clone();
        Ok(())
    }

    pub fn can_be_changed_to(&self, to: T) -> bool {
        self.transitions
            .iter()
            .any(|transition| transition.from == self.current_state && transition.to == to)
    }
}
