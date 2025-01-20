use crate::{
    r#trait::PuppeteerReconstruct,
    state::{BaseConfig, PuppeteerBase, TxState, TxStateStatus},
};
use cosmwasm_std::{attr, DepsMut, Reply, Response, StdError, StdResult};
use drop_helpers::{answer::response, query_id::get_query_id};
use neutron_sdk::bindings::msg::{MsgIbcTransferResponse, MsgSubmitTxResponse};
use serde::{de::DeserializeOwned, Serialize};

impl<'a, T, U, Z> PuppeteerBase<'a, T, U, Z>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
    Z: PuppeteerReconstruct + std::fmt::Debug + Serialize + Clone,
{
    pub fn register_kv_query_reply(
        &self,
        deps: DepsMut,
        msg: Reply,
        query_type: U,
    ) -> StdResult<Response> {
        deps.api.debug(&format!(
            "WASMDEBUG: prepare_sudo_payload received: {msg:?}"
        ));
        let query_id = get_query_id(msg.result)?;
        let attrs = vec![attr("query_id", query_id.to_string())];
        self.kv_queries.save(deps.storage, query_id, &query_type)?;
        Ok(response(
            "sudo-kv-query-payload-received",
            "puppeteer-base",
            attrs,
        ))
    }

    pub fn register_delegations_and_balance_query_reply(
        &self,
        deps: DepsMut,
        msg: Reply,
        i: u16,
        query_type: U,
    ) -> StdResult<Response> {
        let query_id = get_query_id(msg.result)?;
        self.delegations_and_balances_query_id_chunk
            .save(deps.storage, query_id, &i)?;
        self.kv_queries.save(deps.storage, query_id, &query_type)?;
        Ok(Response::new())
    }

    pub fn register_unbonding_delegations_query_reply(
        &self,
        deps: DepsMut,
        msg: Reply,
        validator_index: u16,
        query_type: U,
    ) -> StdResult<Response> {
        deps.api.debug(&format!(
            "WASMDEBUG: register_unbonding_delegations_icq call: {msg:?}",
        ));

        let mut unbonding_delegation = self
            .unbonding_delegations_reply_id_storage
            .load(deps.storage, validator_index)?;
        self.unbonding_delegations_reply_id_storage
            .remove(deps.storage, validator_index);

        let query_id = get_query_id(msg.result)?;
        unbonding_delegation.query_id = query_id;
        self.unbonding_delegations.save(
            deps.storage,
            unbonding_delegation.validator_address.as_ref(),
            &unbonding_delegation,
        )?;

        self.kv_queries.save(deps.storage, query_id, &query_type)?;
        Ok(Response::new())
    }

    pub fn submit_tx_reply(&self, deps: DepsMut, msg: Reply) -> StdResult<Response> {
        let resp: MsgSubmitTxResponse = {
            let result = msg
                .result
                .into_result()
                .map_err(StdError::generic_err)?;

            if let Some(msg_response) = result.msg_responses.first() {
                serde_json_wasm::from_slice(msg_response.value.as_slice())
                    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?
            } else {
                #[allow(deprecated)]
                let data = result
                    .data
                    .ok_or_else(|| StdError::generic_err("no result"))?;
                serde_json_wasm::from_slice(data.as_slice())
                    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?
            }
        };

        deps.api
            .debug(format!("WASMDEBUG: prepare_sudo_payload received; resp: {resp:?}").as_str());
        let seq_id = resp.sequence_id;
        let channel_id = resp.channel;
        let mut self_tx_state: TxState = self.tx_state.load(deps.storage)?;
        self_tx_state.seq_id = Some(seq_id);
        self_tx_state.status = TxStateStatus::WaitingForAck;
        self.tx_state.save(deps.storage, &self_tx_state)?;
        let atts = vec![
            attr("channel_id", channel_id.to_string()),
            attr("seq_id", seq_id.to_string()),
        ];
        Ok(response("sudo-tx-payload-received", "puppeteer-base", atts))
    }

    pub fn submit_ibc_transfer_reply(&self, deps: DepsMut, msg: Reply) -> StdResult<Response> {
        let resp: MsgIbcTransferResponse = {
            let result = msg
                .result
                .into_result()
                .map_err(StdError::generic_err)?;

            if let Some(msg_response) = result.msg_responses.first() {
                serde_json_wasm::from_slice(msg_response.value.as_slice())
                    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?
            } else {
                #[allow(deprecated)]
                let data = result
                    .data
                    .ok_or_else(|| StdError::generic_err("no result"))?;
                serde_json_wasm::from_slice(data.as_slice())
                    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?
            }
        };

        deps.api
            .debug(format!("WASMDEBUG: prepare_sudo_payload received; resp: {resp:?}").as_str());
        let seq_id = resp.sequence_id;
        let channel_id = resp.channel;
        let mut self_tx_state: TxState = self.tx_state.load(deps.storage)?;
        self_tx_state.seq_id = Some(seq_id);
        self_tx_state.status = TxStateStatus::WaitingForAck;
        self.tx_state.save(deps.storage, &self_tx_state)?;
        let atts = vec![
            attr("channel_id", channel_id.to_string()),
            attr("seq_id", seq_id.to_string()),
        ];
        Ok(response(
            "sudo-ibc-transfer-payload-received",
            "puppeteer-base",
            atts,
        ))
    }
}
