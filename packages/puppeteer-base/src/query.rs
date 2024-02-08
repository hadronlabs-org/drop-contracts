use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};
use neutron_sdk::bindings::query::NeutronQuery;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::{ContractError, ContractResult},
    msg::QueryMsg,
    state::{BaseConfig, PuppeteerBase, State, Transfer},
};

impl<'a, T, U> PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub fn query<X: std::fmt::Debug + JsonSchema>(
        &self,
        deps: Deps<NeutronQuery>,
        env: Env,
        msg: QueryMsg<X>,
    ) -> ContractResult<Binary> {
        match msg {
            QueryMsg::State {} => self.query_state(deps, env),
            QueryMsg::Config {} => self.query_config(deps, env),
            QueryMsg::Transactions {} => self.query_transactions(deps, env),
            QueryMsg::Extention { msg } => Err(ContractError::Std(StdError::generic_err(format!(
                "Unsupported query message: {:?}",
                msg
            )))),
        }
    }

    fn query_state(&self, deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
        let state: State = self.state.load(deps.storage)?;
        Ok(to_json_binary(&state)?)
    }

    fn query_config(&self, deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
        let config: T = self.config.load(deps.storage)?;
        Ok(to_json_binary(&config)?)
    }

    fn query_transactions(&self, deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
        let transfers: Vec<Transfer> = self.recipient_transfers.load(deps.storage)?;
        Ok(to_json_binary(&transfers)?)
    }
}
