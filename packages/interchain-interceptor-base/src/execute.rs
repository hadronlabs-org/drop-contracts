use cosmwasm_std::{DepsMut, Response};
use neutron_sdk::NeutronResult;
use serde::{de::DeserializeOwned, Serialize};

use crate::state::{HasOwner, InterchainIntercaptorBase, State};

impl<'a, T, C> InterchainIntercaptorBase<'a, T, C>
where
    T: HasOwner + Serialize + DeserializeOwned + Clone,
    C: std::fmt::Debug + Serialize + DeserializeOwned + Clone,
{
    pub fn instantiate(&self, deps: DepsMut, config: &T) -> NeutronResult<Response> {
        deps.api.debug("WASMDEBUG: instantiate");
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(config.owner()))?;
        self.config.save(deps.storage, config)?;
        self.state.save(deps.storage, &State::default())?;
        self.recipient_txs.save(deps.storage, &vec![])?;
        self.transactions.save(deps.storage, &vec![])?;
        Ok(Response::default())
    }
}
