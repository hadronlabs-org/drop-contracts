use prost::Message;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, GrpcQuery, QueryRequest, StdError, StdResult, Uint64};
use neutron_sdk::bindings::query::NeutronQuery;
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize}; //types::ProtobufAny

// XXX: cosmos_sdk_proto defines these structures for me,
// yet they don't derive serde::de::DeserializeOwned,
// so I have to redefine them here manually >:(

#[cw_serde]
pub struct Fraction {
    pub numerator: Uint64,
    pub denominator: Uint64,
}

#[cw_serde]
pub struct InnerSpec {
    pub child_order: Vec<i32>,
    pub child_size: i32,
    pub min_prefix_length: i32,
    pub max_prefix_length: i32,
    pub empty_child: Option<Vec<u8>>,
    pub hash: String,
}

#[cw_serde]
pub struct LeafOp {
    pub hash: String,
    pub prehash_key: String,
    pub prehash_value: String,
    pub length: String,
    pub prefix: String,
}

#[cw_serde]
pub struct ProofSpec {
    pub leaf_spec: Option<LeafOp>,
    pub inner_spec: Option<InnerSpec>,
    pub max_depth: i32,
    pub min_depth: i32,
    pub prehash_key_before_comparison: bool,
}

#[cw_serde]
pub struct ClientState {
    #[serde(rename = "@type")]
    pub type_url: String,
    pub chain_id: String,
    pub trust_level: Fraction,
    pub trusting_period: Option<String>,
    pub unbonding_period: Option<String>,
    pub max_clock_drift: Option<String>,
    pub frozen_height: Option<Height>,
    pub latest_height: Option<Height>,
    pub proof_specs: Vec<ProofSpec>,
    pub upgrade_path: Vec<String>,
    pub allow_update_after_expiry: bool,
    pub allow_update_after_misbehaviour: bool,
}

#[cw_serde]
pub struct Height {
    pub revision_number: Uint64,
    pub revision_height: Uint64,
}

#[cw_serde]
pub struct IdentifiedClientState {
    pub client_id: String,
    pub client_state: ClientState,
}
#[cw_serde]
pub struct ChannelClientStateResponse {
    pub identified_client_state: Option<IdentifiedClientState>,
    pub proof: Option<Vec<u8>>,
    pub proof_height: Height,
}

pub fn query_client_state(
    deps: &Deps<NeutronQuery>,
    channel_id: String,
    port_id: String,
) -> StdResult<ChannelClientStateResponse> {
    let state = deps.querier
            .query(&QueryRequest::Grpc(GrpcQuery {
                path: "/ibc.core.channel.v1.Query/ChannelClientState".to_string(),
                data: cosmos_sdk_proto::ibc::core::channel::v1::QueryChannelClientStateRequest {
                    port_id: port_id.clone(),
                    channel_id: channel_id.clone(),
                }
                .encode_to_vec()
                .into(),
            }))
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Query channel state for channel {channel_id} and port {port_id} failed: {e}, perhaps, this is wrong channel_id/port_id?"
                ))
            });

    deps.api
        .debug(&format!("WASMDEBUG: query_client_state: {state:?}"));

    state
}
