use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};
use neutron_sdk::bindings::query::NeutronQuery;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    msg::{DelegationsResponse, QueryMsg},
    state::{BaseConfig, PuppeteerBase, State, Transfer},
};

impl<'a, T, C> PuppeteerBase<'a, T, C>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    C: std::fmt::Debug + Serialize + DeserializeOwned + Clone,
{
    pub fn query(&self, deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::State {} => self.query_state(deps, env),
            QueryMsg::Config {} => self.query_config(deps, env),
            QueryMsg::Transactions {} => self.query_transactions(deps, env),
            QueryMsg::InterchainTransactions {} => self.query_done_transactions(deps, env),
            QueryMsg::Delegations {} => self.query_delegations(deps, env),
        }
    }

    fn query_delegations(&self, deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
        let (delegations, last_updated_height) = self.delegations.load(deps.storage)?;
        let response = DelegationsResponse {
            delegations,
            last_updated_height,
        };
        to_json_binary(&response)
    }

    fn query_state(&self, deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
        let state: State = self.state.load(deps.storage)?;
        to_json_binary(&state)
    }

    fn query_done_transactions(&self, deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
        deps.api.debug("WASMDEBUG: query_done_transactions");
        let state: Vec<C> = self.transactions.load(deps.storage)?;
        to_json_binary(&state)
    }

    fn query_config(&self, deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
        let config: T = self.config.load(deps.storage)?;
        to_json_binary(&config)
    }

    fn query_transactions(&self, deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
        let transactions: Vec<Transfer> = self.recipient_txs.load(deps.storage)?;
        to_json_binary(&transactions)
    }
}
