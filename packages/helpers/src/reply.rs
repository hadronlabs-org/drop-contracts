use cosmwasm_std::{StdError, StdResult, SubMsgResult};
use neutron_std::types::neutron::interchainqueries::MsgRegisterInterchainQueryResponse;

pub fn get_query_id(msg_result: SubMsgResult) -> StdResult<u64> {
    msg_result
        .into_result()
        .map_err(StdError::generic_err)?
        .data
        .ok_or_else(|| StdError::generic_err("no result"))
        .and_then(|data| {
            decode_message_response::<MsgRegisterInterchainQueryResponse>(&data.to_vec())
                .map_err(|e| StdError::generic_err(format!("failed to decode response in get_query_id: {e:?}")))
        })
        .map(|res| res.id)
}
