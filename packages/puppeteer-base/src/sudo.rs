use crate::{
    msg::OpenAckVersion,
    r#trait::PuppeteerReconstruct,
    state::{BaseConfig, PuppeteerBase, Transfer},
};
use cosmos_sdk_proto::cosmos::{
    bank::v1beta1::MsgSend,
    tx::v1beta1::{TxBody, TxRaw},
};
use cosmwasm_std::{Binary, DepsMut, Env, Response, StdError, Timestamp};
use cw_storage_plus::Index;
use neutron_sdk::{
    bindings::{
        query::{NeutronQuery, QueryRegisteredQueryResponse},
        types::Height,
    },
    interchain_queries::{
        get_registered_query,
        queries::get_raw_interchain_query_result,
        types::QueryType,
        v045::{queries::query_unbonding_delegations, types::COSMOS_SDK_TRANSFER_MSG_URL},
    },
    NeutronError, NeutronResult,
};
use prost::Message;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, T, U> PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub fn sudo_tx_query_result(
        &self,
        deps: DepsMut<NeutronQuery>,
        _env: Env,
        query_id: u64,
        _height: Height,
        data: Binary,
    ) -> NeutronResult<Response> {
        let _config: T = self.config.load(deps.storage)?;
        let tx: TxRaw = TxRaw::decode(data.as_slice())?;
        let body: TxBody = TxBody::decode(tx.body_bytes.as_slice())?;
        let registered_query: QueryRegisteredQueryResponse =
            get_registered_query(deps.as_ref(), query_id)?;
        #[allow(clippy::single_match)]
        match registered_query.registered_query.query_type {
            QueryType::TX => {
                let ica = self.ica.get_address(deps.storage)?;
                let deposits = self.recipient_deposits_from_tx_body(body, &ica)?;
                if deposits.is_empty() {
                    return Err(NeutronError::Std(StdError::generic_err(
                        "failed to find a matching transaction message",
                    )));
                }
                let mut txs = self.recipient_transfers.load(deps.storage)?;
                txs.extend(deposits);
                self.recipient_transfers.save(deps.storage, &txs)?;
            }
            _ => {}
        }
        Ok(Response::new())
    }

    /// parses tx body and retrieves transactions to the given recipient.
    fn recipient_deposits_from_tx_body(
        &self,
        tx_body: TxBody,
        recipient: &str,
    ) -> NeutronResult<Vec<Transfer>> {
        let mut deposits: Vec<Transfer> = vec![];
        // for msg in tx_body.messages.iter().take(MAX_ALLOWED_MESSAGES) {
        for msg in tx_body.messages.iter() {
            #[allow(clippy::single_match)]
            match msg.type_url.as_str() {
                COSMOS_SDK_TRANSFER_MSG_URL => {
                    // Parse a Send message and check that it has the required recipient.
                    let transfer_msg: MsgSend = MsgSend::decode(msg.value.as_slice())?;
                    if transfer_msg.to_address == recipient {
                        for coin in transfer_msg.amount {
                            deposits.push(Transfer {
                                sender: transfer_msg.from_address.clone(),
                                amount: coin.amount.clone(),
                                denom: coin.denom,
                                recipient: recipient.to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(deposits)
    }

    pub fn sudo_kv_query_result<
        X: PuppeteerReconstruct + std::fmt::Debug + Serialize + Clone + DeserializeOwned,
    >(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
        query_id: u64,
        version: &str,
        storage: cw_storage_plus::Item<'a, (X, u64, Timestamp)>,
    ) -> NeutronResult<Response> {
        let registered_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;
        deps.api.debug(&format!(
            "WASMDEBUG: sudo_kv_query_result: registered_query_result: {:?}",
            registered_query_result
        ));
        let data =
            PuppeteerReconstruct::reconstruct(&registered_query_result.result.kv_results, version)?;

        let height = env.block.height;
        let timestamp = env.block.time;
        storage.save(deps.storage, &(data, height, timestamp))?;
        Ok(Response::default())
    }

    pub fn sudo_unbonding_delegations_kv_query_result(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
        query_id: u64,
    ) -> NeutronResult<Response> {
        if let Some(mut item) = self
            .unbonding_delegations
            .idx
            .query_id
            .item(deps.storage, query_id)?
        {
            self.unbonding_delegations
                .idx
                .query_id
                .remove(deps.storage, &item.0, &item.1)?;

            item.1.unbonding_delegations =
                query_unbonding_delegations(deps.as_ref(), env.clone(), query_id)?
                    .unbonding_delegations
                    .unbonding_responses
                    .pop()
                    .unwrap()
                    .entries;
            item.1.last_updated_height = env.block.height;

            self.unbonding_delegations
                .save(deps.storage, &item.1.validator_address, &item.1)?;
        }

        Ok(Response::default())
    }

    pub fn sudo_open_ack(
        &self,
        deps: DepsMut<NeutronQuery>,
        _env: Env,
        _port_id: String,
        _channel_id: String,
        _counterparty_channel_id: String,
        counterparty_version: String,
    ) -> NeutronResult<Response> {
        let parsed_version: Result<OpenAckVersion, _> =
            serde_json_wasm::from_str(counterparty_version.as_str());
        if let Ok(parsed_version) = parsed_version {
            self.ica.set_address(deps.storage, parsed_version.address)?;
            Ok(Response::default())
        } else {
            Err(NeutronError::Std(StdError::generic_err(
                "can't parse version",
            )))
        }
    }
}
