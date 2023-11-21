use cosmwasm_std::{Coin as CosmosCoin, DepsMut, Env, Response, StdError, Uint128};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    interchain_queries::v045::{
        new_register_delegator_delegations_query_msg, new_register_transfers_query_msg,
    },
    interchain_txs::helpers::get_port_id,
    NeutronError, NeutronResult,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    msg::ExecuteMsg,
    state::{BaseConfig, InterchainIntercaptorBase, State, ICA_ID, LOCAL_DENOM},
};

impl<'a, T, C> InterchainIntercaptorBase<'a, T, C>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
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

    pub fn execute(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
        msg: ExecuteMsg,
    ) -> NeutronResult<Response<NeutronMsg>> {
        match msg {
            ExecuteMsg::RegisterICA {} => self.execute_register_ica(deps, env),
            ExecuteMsg::RegisterQuery {} => self.register_transfers_query(deps),
            ExecuteMsg::RegisterDelegatorDelegationsQuery { validators } => {
                self.register_delegations_query(deps, validators)
            }
            ExecuteMsg::SetFees {
                recv_fee,
                ack_fee,
                timeout_fee,
            } => self.execute_set_fees(deps, recv_fee, ack_fee, timeout_fee),
        }
    }

    fn execute_register_ica(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
    ) -> NeutronResult<Response<NeutronMsg>> {
        let config = self.config.load(deps.storage)?;
        let state: State = self.state.load(deps.storage)?;
        match state.ica {
            None => {
                let register = NeutronMsg::register_interchain_account(
                    config.connection_id(),
                    ICA_ID.to_string(),
                );
                let _key = get_port_id(env.contract.address.as_str(), ICA_ID);

                Ok(Response::new().add_message(register))
            }
            Some(_) => Err(NeutronError::Std(cosmwasm_std::StdError::GenericErr {
                msg: "ICA already registered".to_string(),
            })),
        }
    }

    fn execute_set_fees(
        &self,
        deps: DepsMut<NeutronQuery>,
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
    ) -> NeutronResult<Response<NeutronMsg>> {
        let fees = IbcFee {
            recv_fee: vec![CosmosCoin {
                denom: LOCAL_DENOM.to_string(),
                amount: recv_fee,
            }],
            ack_fee: vec![CosmosCoin {
                denom: LOCAL_DENOM.to_string(),
                amount: ack_fee,
            }],
            timeout_fee: vec![CosmosCoin {
                denom: LOCAL_DENOM.to_string(),
                amount: timeout_fee,
            }],
        };
        self.ibc_fee.save(deps.storage, &fees)?;
        Ok(Response::default())
    }

    fn register_transfers_query(
        &self,
        deps: DepsMut<NeutronQuery>,
    ) -> NeutronResult<Response<NeutronMsg>> {
        let config = self.config.load(deps.storage)?;
        let state: State = self.state.load(deps.storage)?;

        if let Some(ica) = state.ica {
            let msg = new_register_transfers_query_msg(
                config.connection_id(),
                ica,
                config.update_period(),
                None,
            )?;
            Ok(Response::new().add_message(msg))
        } else {
            Err(NeutronError::IntegrationTestsMock {})
        }
    }

    fn register_delegations_query(
        &self,
        deps: DepsMut<NeutronQuery>,
        validators: Vec<String>,
    ) -> NeutronResult<Response<NeutronMsg>> {
        let config = self.config.load(deps.storage)?;
        let state: State = self.state.load(deps.storage)?;

        let delegator = state.ica.ok_or_else(|| {
            StdError::generic_err("Interchain account is not registered. Please register it first")
        })?;
        let msg = new_register_delegator_delegations_query_msg(
            config.connection_id(),
            delegator,
            validators,
            config.update_period(),
        )?;
        Ok(Response::new().add_message(msg))
    }
}
