use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};
use neutron_sdk::bindings::query::NeutronQuery;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::state::ConfigResponse;
use crate::{
    error::{ContractError, ContractResult},
    msg::QueryMsg,
    state::{BaseConfig, PuppeteerBase, Transfer},
};

impl<'a, T, U> PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub fn query<X: std::fmt::Debug + JsonSchema>(
        &self,
        deps: Deps<NeutronQuery>,
        _env: Env,
        msg: QueryMsg<X>,
    ) -> ContractResult<Binary> {
        match msg {
            QueryMsg::Config {} => self.query_config(deps),
            QueryMsg::Ica {} => self.query_ica(deps),
            QueryMsg::Transactions {} => self.query_transactions(deps),
            QueryMsg::Extention { msg } => Err(ContractError::Std(StdError::generic_err(format!(
                "Unsupported query message: {:?}",
                msg
            )))),
        }
    }

    fn query_ica(&self, deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
        let ica_state = self.ica.load(deps.storage)?;
        Ok(to_json_binary(&ica_state)?)
    }

    fn query_config(&self, deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
        let config: T = self.config.load(deps.storage)?;
        Ok(to_json_binary(&ConfigResponse {
            owner: config.owner().to_string(),
            connection_id: config.connection_id(),
            update_period: config.update_period(),
        })?)
    }

    fn query_transactions(&self, deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
        let transfers: Vec<Transfer> = self.recipient_transfers.load(deps.storage)?;
        Ok(to_json_binary(&transfers)?)
    }
}
