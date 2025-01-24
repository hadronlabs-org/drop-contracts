use cosmwasm_std::{StdError, StdResult, SubMsgResult};
use neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse;
use serde_json_wasm::from_slice;

pub fn get_query_id(msg_result: SubMsgResult) -> StdResult<u64> {
    msg_result
        .into_result()
        .map_err(StdError::generic_err)?
        .msg_responses
        .first()
        .ok_or_else(|| StdError::generic_err("no msg_responses found"))
        .and_then(|msg_response| {
            from_slice::<MsgRegisterInterchainQueryResponse>(msg_response.value.as_slice())
                .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))
        })
        .map(|res| res.id)
}
