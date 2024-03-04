use cosmwasm_std::{
    attr, ensure_eq, Coin as CosmosCoin, CosmosMsg, CustomQuery, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, SubMsg, Uint128,
};
use lido_helpers::answer::response;
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    interchain_queries::v045::new_register_transfers_query_msg,
    NeutronError, NeutronResult,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, Transaction},
    state::{BaseConfig, PuppeteerBase, TxState, TxStateStatus, ICA_ID, LOCAL_DENOM},
};

impl<'a, T, U> PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub fn instantiate(&self, deps: DepsMut, config: &T) -> NeutronResult<Response> {
        deps.api.debug("WASMDEBUG: instantiate");
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(config.owner()))?;

        self.config.save(deps.storage, config)?;
        self.recipient_transfers.save(deps.storage, &vec![])?;
        Ok(Response::default())
    }

    pub fn execute(
        &self,
        deps: DepsMut<NeutronQuery>,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> ContractResult<Response<NeutronMsg>> {
        match msg {
            ExecuteMsg::RegisterICA {} => self.execute_register_ica(deps),
            ExecuteMsg::RegisterQuery {} => self.register_transfers_query(deps),
            ExecuteMsg::SetFees {
                recv_fee,
                ack_fee,
                timeout_fee,
                register_fee,
            } => self.execute_set_fees(deps, info, recv_fee, ack_fee, timeout_fee, register_fee),
        }
    }

    pub fn validate_tx_state<C: CustomQuery>(
        &self,
        deps: Deps<C>,
        status: TxStateStatus,
    ) -> NeutronResult<()> {
        let tx_state = self.tx_state.load(deps.storage).unwrap_or_default();
        deps.api.debug(&format!(
            "WASMDEBUG: validate_tx_state: real state: {:?} checked state: {:?}",
            tx_state, status
        ));
        ensure_eq!(
            tx_state.status,
            status,
            NeutronError::Std(StdError::generic_err(format!(
                "Transaction txState is not equal to expected: {}",
                status
            )))
        );
        Ok(())
    }

    pub fn validate_tx_idle_state<C: CustomQuery>(&self, deps: Deps<C>) -> NeutronResult<()> {
        self.validate_tx_state(deps, TxStateStatus::Idle)
    }

    pub fn validate_tx_waiting_state<C: CustomQuery>(&self, deps: Deps<C>) -> NeutronResult<()> {
        self.validate_tx_state(deps, TxStateStatus::WaitingForAck)
    }

    pub fn validate_tx_inprogress_state<C: CustomQuery>(&self, deps: Deps<C>) -> NeutronResult<()> {
        self.validate_tx_state(deps, TxStateStatus::InProgress)
    }

    pub fn msg_with_sudo_callback<C: Into<CosmosMsg<X>> + Serialize, X>(
        &self,
        deps: DepsMut<NeutronQuery>,
        msg: C,
        transaction: Transaction,
        reply_to: String,
        payload_id: u64,
    ) -> StdResult<SubMsg<X>> {
        deps.api
            .debug("WASMDEBUG: msg_with_sudo_callback save tx_state InProgress");
        self.tx_state.save(
            deps.storage,
            &TxState {
                status: TxStateStatus::InProgress,
                seq_id: None,
                transaction: Some(transaction),
                reply_to: Some(reply_to),
            },
        )?;
        Ok(SubMsg::reply_on_success(msg, payload_id))
    }

    fn execute_register_ica(
        &self,
        deps: DepsMut<NeutronQuery>,
    ) -> ContractResult<Response<NeutronMsg>> {
        let config = self.config.load(deps.storage)?;
        let attrs = vec![
            attr("connection_id", config.connection_id()),
            attr("ica_id", ICA_ID),
        ];
        let register_fee = self.register_fee.load(deps.storage)?;
        let register_msg =
            self.ica
                .register(deps.storage, config.connection_id(), ICA_ID, register_fee)?;
        Ok(response("register-ica", "puppeteer-base", attrs).add_message(register_msg))
    }

    fn execute_set_fees(
        &self,
        deps: DepsMut<NeutronQuery>,
        info: MessageInfo,
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
        register_fee: Uint128,
    ) -> ContractResult<Response<NeutronMsg>> {
        let config = self.config.load(deps.storage)?;
        ensure_eq!(
            config.owner(),
            info.sender.as_str(),
            ContractError::Unauthorized {}
        );
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
        let ica = self.ica.get_address(deps.storage)?;

        let msg = new_register_transfers_query_msg(
            config.connection_id(),
            ica,
            config.update_period(),
            None,
        )?;
        Ok(Response::new().add_message(msg))
    }
}
