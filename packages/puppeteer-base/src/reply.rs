use cosmwasm_std::{attr, DepsMut, Env, Reply, Response, StdError, StdResult};
use lido_staking_base::helpers::answer::response;
use neutron_sdk::bindings::msg::MsgSubmitTxResponse;
use serde::{de::DeserializeOwned, Serialize};

use crate::state::{BaseConfig, PuppeteerBase, TxState, TxStateStatus, SUDO_PAYLOAD_REPLY_ID};

impl<'a, T> PuppeteerBase<'a, T>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
{
    pub fn reply(&self, deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
        deps.api
            .debug(format!("WASMDEBUG: reply msg: {msg:?}").as_str());
        match msg.id {
            SUDO_PAYLOAD_REPLY_ID => self.update_tx_state(deps, env, msg),
            _ => Err(StdError::generic_err(format!(
                "unsupported reply message id {}",
                msg.id
            ))),
        }
    }

    fn update_tx_state(&self, deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
        let resp: MsgSubmitTxResponse = serde_json_wasm::from_slice(
            msg.result
                .into_result()
                .map_err(StdError::generic_err)?
                .data
                .ok_or_else(|| StdError::generic_err("no result"))?
                .as_slice(),
        )
        .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?;
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
        Ok(response("sudo-payload-received", "puppeteer-base", atts))
    }
}
