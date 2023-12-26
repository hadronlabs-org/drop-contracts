use cosmwasm_std::{
    attr, Coin as CosmosCoin, CosmosMsg, DepsMut, Env, Response, StdError, StdResult, SubMsg,
    Uint128,
};
use lido_staking_base::helpers::answer::response;
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    interchain_queries::v045::new_register_transfers_query_msg,
    interchain_txs::helpers::get_port_id,
    NeutronError, NeutronResult,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, Transaction},
    state::{
        BaseConfig, IcaState, PuppeteerBase, State, TxState, TxStateStatus, ICA_ID, LOCAL_DENOM,
        SUDO_PAYLOAD_REPLY_ID,
    },
};

impl<'a, T> PuppeteerBase<'a, T>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
{
    pub fn instantiate(&self, deps: DepsMut, config: &T) -> NeutronResult<Response> {
        deps.api.debug("WASMDEBUG: instantiate");
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(config.owner()))?;

        self.config.save(deps.storage, config)?;
        self.state.save(deps.storage, &State::default())?;
        self.recipient_txs.save(deps.storage, &vec![])?;
        Ok(Response::default())
    }

    pub fn execute(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
        msg: ExecuteMsg,
    ) -> ContractResult<Response<NeutronMsg>> {
        match msg {
            ExecuteMsg::RegisterICA {} => self.execute_register_ica(deps, env),
            ExecuteMsg::RegisterQuery {} => self.register_transfers_query(deps),
            ExecuteMsg::SetFees {
                recv_fee,
                ack_fee,
                timeout_fee,
                register_fee,
            } => self.execute_set_fees(deps, recv_fee, ack_fee, timeout_fee, register_fee),
        }
    }

    pub fn get_ica(&self, state: &State) -> NeutronResult<String> {
        match state.ica_state {
            IcaState::None => Err(NeutronError::Std(StdError::generic_err(
                "Interchain account is not registered",
            ))),
            IcaState::InProgress => Err(NeutronError::Std(StdError::generic_err(
                "Interchain account is in progress. Please wait until it is registered",
            ))),
            IcaState::Registered => state.clone().ica.ok_or_else(|| {
                NeutronError::Std(StdError::generic_err(
                    "Interchain account is not registered. Please register it first",
                ))
            }),
            IcaState::Timeout => Err(NeutronError::Std(StdError::generic_err(
                "Interchain account registration timeout. Please register it again",
            ))),
        }
    }

    pub fn msg_with_sudo_callback<C: Into<CosmosMsg<X>> + Serialize, X>(
        &self,
        deps: DepsMut<NeutronQuery>,
        msg: C,
        transaction: Transaction,
        reply_to: String,
    ) -> StdResult<SubMsg<X>> {
        self.tx_state.save(
            deps.storage,
            &TxState {
                status: TxStateStatus::InProgress,
                seq_id: None,
                transaction: Some(transaction),
                reply_to: Some(reply_to),
            },
        )?;
        Ok(SubMsg::reply_on_success(msg, SUDO_PAYLOAD_REPLY_ID))
    }

    fn execute_register_ica(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
    ) -> ContractResult<Response<NeutronMsg>> {
        let config = self.config.load(deps.storage)?;
        let state: State = self.state.load(deps.storage)?;
        let attrs = vec![
            attr("connection_id", config.connection_id()),
            attr("ica_id", ICA_ID),
        ];
        return match state.ica_state {
            IcaState::InProgress => Err(ContractError::IcaInProgress {}),
            IcaState::Registered => Err(ContractError::IcaAlreadyRegistered {}),
            IcaState::Timeout | IcaState::None => {
                let register_fee = self.register_fee.load(deps.storage)?;
                let register = NeutronMsg::register_interchain_account(
                    config.connection_id(),
                    ICA_ID.to_string(),
                    Some(vec![register_fee]),
                );
                let _key = get_port_id(env.contract.address.as_str(), ICA_ID);

                self.state.save(
                    deps.storage,
                    &State {
                        last_processed_height: None,
                        ica: None,
                        ica_state: IcaState::InProgress,
                    },
                )?;
                Ok(response("register-ica", "puppeteer-base", attrs).add_message(register))
            }
        };
    }

    fn execute_set_fees(
        &self,
        deps: DepsMut<NeutronQuery>,
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
        register_fee: Uint128,
    ) -> ContractResult<Response<NeutronMsg>> {
        // TODO: Change LOCAL_DENOM to configurable value
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
        self.register_fee.save(
            deps.storage,
            &CosmosCoin {
                amount: register_fee,
                denom: LOCAL_DENOM.to_string(),
            },
        )?;
        Ok(Response::default())
    }

    fn register_transfers_query(
        &self,
        deps: DepsMut<NeutronQuery>,
    ) -> ContractResult<Response<NeutronMsg>> {
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
            Err(ContractError::IcaNotRegistered {})
        }
    }
}
