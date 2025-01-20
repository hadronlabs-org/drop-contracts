use cosmwasm_std::{StdError, StdResult, SubMsgResult};
use neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse;
use serde_json_wasm::from_slice;

pub fn get_query_id(msg_result: SubMsgResult) -> StdResult<u64> {
    let result = msg_result.into_result().map_err(StdError::generic_err)?;

    if let Some(msg_response) = result.msg_responses.first() {
        let res: MsgRegisterInterchainQueryResponse =
            from_slice(msg_response.value.as_slice())
                .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?;
        return Ok(res.id);
    }

    #[allow(deprecated)]
    if let Some(data) = result.data {
        let res: MsgRegisterInterchainQueryResponse =
            from_slice(data.as_slice())
                .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?;
        return Ok(res.id);
    }

    Err(StdError::generic_err("no data or msg_responses found"))
}

