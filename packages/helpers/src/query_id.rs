use cosmwasm_std::{StdError, StdResult, SubMsgResult};
use neutron_sdk::interchain_txs::helpers::decode_message_response;
use neutron_std::types::neutron::interchainqueries::MsgRegisterInterchainQueryResponse;

pub fn get_query_id(msg_result: SubMsgResult) -> StdResult<u64> {
    msg_result
        .into_result()
        .map_err(StdError::generic_err)?
        .msg_responses
        .first()
        .ok_or_else(|| StdError::generic_err("no msg_response found in result"))
        .and_then(|msg_response| {
            decode_message_response::<MsgRegisterInterchainQueryResponse>(
                &msg_response.value.to_vec(),
            )
            .map_err(|e| {
                StdError::generic_err(format!("failed to decode response in query_id: {e:?}"))
            })
        })
        .map(|res| res.id)
}
