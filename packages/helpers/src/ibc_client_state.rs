use prost::Message;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, QueryRequest, StdError, StdResult};
use neutron_sdk::bindings::query::NeutronQuery;

// XXX: cosmos_sdk_proto defines these structures for me,
// yet they don't derive serde::de::DeserializeOwned,
// so I have to redefine them here manually >:(
#[cw_serde]
pub struct Height {
    pub revision_number: u64,
    pub revision_height: u64,
}
#[cw_serde]
pub struct ChannelClientStateResponse {
    pub proof_height: Option<Height>,
}

pub fn query_client_state(
    deps: &Deps<NeutronQuery>,
    channel_id: String,
    port_id: String,
) -> StdResult<ChannelClientStateResponse> {
    let state = deps.querier
            .query(&QueryRequest::Stargate {
                path: "/ibc.core.channel.v1.Query/ChannelClientState".to_string(),
                data: cosmos_sdk_proto::ibc::core::channel::v1::QueryChannelClientStateRequest {
                    port_id: port_id.clone(),
                    channel_id: channel_id.clone(),
                }
                    .encode_to_vec()
                    .into(),
            })
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Query channel state for channel {channel_id} and port {port_id} failed: {e}, perhaps, this is wrong channel_id/port_id?"
                ))
            });

    deps.api
        .debug(&format!("WASMDEBUG: query_client_state: {state:?}"));

    state
}
