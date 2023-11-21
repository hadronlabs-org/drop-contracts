use cosmwasm_std::{from_binary, Binary, DepsMut, Env, Reply, Response, StdError, StdResult};
use neutron_sdk::bindings::msg::MsgSubmitTxResponse;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    msg::SudoPayload,
    state::{BaseConfig, InterchainIntercaptorBase, SUDO_PAYLOAD_REPLY_ID},
};

impl<'a, T, C> InterchainIntercaptorBase<'a, T, C>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    C: std::fmt::Debug + Serialize + DeserializeOwned + Clone,
{
    pub fn reply(&self, deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
        deps.api
            .debug(format!("WASMDEBUG: reply msg: {msg:?}").as_str());
        match msg.id {
            SUDO_PAYLOAD_REPLY_ID => self.prepare_sudo_payload(deps, env, msg),
            _ => Err(StdError::generic_err(format!(
                "unsupported reply message id {}",
                msg.id
            ))),
        }
    }

    fn prepare_sudo_payload(&self, deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
        let data = self.reply_id_storage.load(deps.storage)?;
        let payload: SudoPayload<C> = from_binary(&Binary(data))?;
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
        self.sudo_payload
            .save(deps.storage, (channel_id, seq_id), &payload)?;
        Ok(Response::new())
    }
}
