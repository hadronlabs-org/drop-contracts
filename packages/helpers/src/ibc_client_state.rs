use cosmwasm_std::{Deps, StdError, StdResult};
use neutron_sdk::bindings::query::NeutronQuery;
use prost::Message;

// XXX: cosmos_sdk_proto defines these structures for me,
// yet they don't derive serde::de::DeserializeOwned,
// so I have to redefine them here manually.

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fraction {
    #[prost(uint64, tag = "1")]
    pub numerator: u64,
    #[prost(uint64, tag = "2")]
    pub denominator: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InnerSpec {
    #[prost(int32, repeated, tag = "1")]
    pub child_order: Vec<i32>,
    #[prost(int32, tag = "2")]
    pub child_size: i32,
    #[prost(int32, tag = "3")]
    pub min_prefix_length: i32,
    #[prost(int32, tag = "4")]
    pub max_prefix_length: i32,
    #[prost(bytes, optional, tag = "5")]
    pub empty_child: Option<Vec<u8>>,
    #[prost(string, tag = "6")]
    pub hash: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LeafOp {
    #[prost(string, tag = "1")]
    pub hash: String,
    #[prost(string, tag = "2")]
    pub prehash_key: String,
    #[prost(string, tag = "3")]
    pub prehash_value: String,
    #[prost(string, tag = "4")]
    pub length: String,
    #[prost(string, tag = "5")]
    pub prefix: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofSpec {
    #[prost(message, optional, tag = "1")]
    pub leaf_spec: Option<LeafOp>,
    #[prost(message, optional, tag = "2")]
    pub inner_spec: Option<InnerSpec>,
    #[prost(int32, tag = "3")]
    pub max_depth: i32,
    #[prost(int32, tag = "4")]
    pub min_depth: i32,
    #[prost(bool, tag = "5")]
    pub prehash_key_before_comparison: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    /// Corresponds to the `@type` field.
    #[prost(string, tag = "1")]
    pub type_url: String,
    #[prost(string, tag = "2")]
    pub chain_id: String,
    /// Marking as optional to match the Protobuf encoding.
    #[prost(message, optional, tag = "3")]
    pub trust_level: Option<Fraction>,
    #[prost(string, optional, tag = "4")]
    pub trusting_period: Option<String>,
    #[prost(string, optional, tag = "5")]
    pub unbonding_period: Option<String>,
    #[prost(string, optional, tag = "6")]
    pub max_clock_drift: Option<String>,
    #[prost(message, optional, tag = "7")]
    pub frozen_height: Option<Height>,
    #[prost(message, optional, tag = "8")]
    pub latest_height: Option<Height>,
    #[prost(message, repeated, tag = "9")]
    pub proof_specs: Vec<ProofSpec>,
    #[prost(string, repeated, tag = "10")]
    pub upgrade_path: Vec<String>,
    #[prost(bool, tag = "11")]
    pub allow_update_after_expiry: bool,
    #[prost(bool, tag = "12")]
    pub allow_update_after_misbehaviour: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Height {
    #[prost(uint64, tag = "1")]
    pub revision_number: u64,
    #[prost(uint64, tag = "2")]
    pub revision_height: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedClientState {
    #[prost(string, tag = "1")]
    pub client_id: String,
    #[prost(message, optional, tag = "2")]
    pub client_state: Option<ClientState>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelClientStateResponse {
    #[prost(message, optional, tag = "1")]
    pub identified_client_state: Option<IdentifiedClientState>,
    #[prost(bytes, optional, tag = "2")]
    pub proof: Option<Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub proof_height: Option<Height>,
}

pub fn query_client_state(
    deps: &Deps<NeutronQuery>,
    channel_id: String,
    port_id: String,
) -> StdResult<ChannelClientStateResponse> {
    let raw = deps
        .querier
        .query_grpc(
            "/ibc.core.channel.v1.Query/ChannelClientState".to_string(),
            cosmos_sdk_proto::ibc::core::channel::v1::QueryChannelClientStateRequest {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            }
                .encode_to_vec()
                .into(),
        )
        .map_err(|e| {
            StdError::generic_err(format!(
                "Query channel state for channel {} and port {} failed: {}, perhaps, this is wrong channel_id/port_id?",
                channel_id, port_id, e
            ))
        })?;

    deps.api
        .debug(&format!("WASMDEBUG: query_client_state raw: {:?}", raw));

    let state = ChannelClientStateResponse::decode(raw.as_slice())
        .map_err(|e| StdError::generic_err(format!("failed to decode protobuf: {:?}", e)))?;

    Ok(state)
}
