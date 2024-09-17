use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, Transaction},
    r#trait::PuppeteerReconstruct,
    state::{BaseConfig, PuppeteerBase, TxState, TxStateStatus, ICA_ID, LOCAL_DENOM},
};
use cosmwasm_std::{
    attr, ensure_eq, CosmosMsg, CustomQuery, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, SubMsg,
};
use drop_helpers::answer::response;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v045::new_register_transfers_query_msg,
    NeutronError, NeutronResult,
};
use serde::{de::DeserializeOwned, Serialize};

impl<'a, T, U, P> PuppeteerBase<'a, T, U, P>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
    P: PuppeteerReconstruct + std::fmt::Debug + Serialize + Clone,
{
    pub fn instantiate<X: CustomQuery, Z>(
        &self,
        deps: DepsMut<X>,
        config: &T,
        owner: String,
    ) -> NeutronResult<Response<Z>> {
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;
        self.config.save(deps.storage, config)?;
        self.recipient_transfers.save(deps.storage, &vec![])?;
        Ok(Response::<Z>::default())
    }

    pub fn execute(
        &self,
        deps: DepsMut<NeutronQuery>,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> ContractResult<Response<NeutronMsg>> {
        match msg {
            ExecuteMsg::RegisterICA {} => self.execute_register_ica(deps, info),
            ExecuteMsg::RegisterQuery {} => self.register_transfers_query(deps),
        }
    }

    pub fn update_config(&self, deps: DepsMut, config: &T) -> NeutronResult<Response> {
        self.config.save(deps.storage, config)?;
        Ok(Response::default())
    }

    pub fn validate_tx_state<C: CustomQuery>(
        &self,
        deps: Deps<C>,
        status: TxStateStatus,
    ) -> NeutronResult<()> {
        let tx_state = self.tx_state.load(deps.storage).unwrap_or_default();
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
        info: MessageInfo,
    ) -> ContractResult<Response<NeutronMsg>> {
        let config = self.config.load(deps.storage)?;
        let attrs = vec![
            attr("connection_id", config.connection_id()),
            attr("ica_id", ICA_ID),
        ];
        let register_fee = info
            .funds
            .into_iter()
            .find(|f| f.denom == LOCAL_DENOM)
            .ok_or(ContractError::InvalidFunds {
                reason: format!("missing fee in denom {}", LOCAL_DENOM),
            })?;
        let register_msg =
            self.ica
                .register(deps.storage, config.connection_id(), ICA_ID, register_fee)?;
        Ok(response("register-ica", "puppeteer-base", attrs).add_message(register_msg))
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
