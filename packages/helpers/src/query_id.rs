use cosmwasm_std::{StdError, StdResult, SubMsgResult};
use neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse;
use serde_json_wasm::from_slice;

pub fn get_query_id(msg_result: SubMsgResult) -> StdResult<u64> {
    let res: MsgRegisterInterchainQueryResponse = from_slice(
        msg_result
            .into_result()
            .map_err(StdError::generic_err)?
            .data
            .ok_or_else(|| StdError::generic_err("no result"))?
            .as_slice(),
    )
    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?;

    Ok(res.id)
}
