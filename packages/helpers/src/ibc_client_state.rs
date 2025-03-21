use cosmwasm_std::{Deps, StdError, StdResult};
use neutron_sdk::bindings::query::NeutronQuery;
use prost::Message;
use prost_types::{Any, Duration};

// These types are re‑defined here because cosmos_sdk_proto types
// don't derive DeserializeOwned, so we use Prost to decode the binary data.

#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub enum HashOp {
    NoHash = 0,
    Sha256 = 1,
    Sha512 = 2,
    Keccak = 3,
    Ripemd160 = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub enum LengthOp {
    NoPrefix = 0,
    VarProto = 1,
    VarRlp = 2,
    Fixed = 3,
}

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
    #[prost(bytes, tag = "5")]
    pub empty_child: Vec<u8>,
    #[prost(enumeration = "HashOp", tag = "6")]
    pub hash: i32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LeafOp {
    #[prost(enumeration = "HashOp", tag = "1")]
    pub hash: i32, // Use i32 for the underlying representation; convert via HashOp::from_i32(...)
    #[prost(enumeration = "HashOp", tag = "2")]
    pub prehash_key: i32,
    #[prost(enumeration = "HashOp", tag = "3")]
    pub prehash_value: i32,
    #[prost(enumeration = "LengthOp", tag = "4")]
    pub length: i32,
    #[prost(bytes, tag = "5")]
    pub prefix: Vec<u8>,
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
    // We omit the original "@type" renaming here; you may choose to include it if needed.
    #[prost(string, tag = "1")]
    pub chain_id: String,
    #[prost(message, tag = "2")]
    pub trust_level: Option<Fraction>,
    #[prost(message, tag = "3")]
    pub trusting_period: Option<Duration>,
    #[prost(message, tag = "4")]
    pub unbonding_period: Option<Duration>,
    #[prost(message, tag = "5")]
    pub max_clock_drift: Option<Duration>,
    #[prost(message, tag = "6")]
    pub frozen_height: Option<Height>,
    #[prost(message, tag = "7")]
    pub latest_height: Option<Height>,
    #[prost(message, repeated, tag = "8")]
    pub proof_specs: Vec<ProofSpec>,
    #[prost(string, repeated, tag = "9")]
    pub upgrade_path: Vec<String>,
    // Deprecated fields; include them if necessary.
    #[prost(bool, tag = "10")]
    pub allow_update_after_expiry: bool,
    #[prost(bool, tag = "11")]
    pub allow_update_after_misbehaviour: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Height {
    #[prost(uint64, tag = "1")]
    pub revision_number: u64,
    #[prost(uint64, tag = "2")]
    pub revision_height: u64,
}

/// According to the IBC spec, IdentifiedClientState contains a google.protobuf.Any.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedClientState {
    #[prost(string, tag = "1")]
    pub client_id: String,
    #[prost(message, optional, tag = "2")]
    pub client_state: Option<Any>,
}

/// The response to the ChannelClientState query.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelClientStateResponse {
    #[prost(message, optional, tag = "1")]
    pub identified_client_state: Option<IdentifiedClientState>,
    #[prost(bytes, optional, tag = "2")]
    pub proof: Option<Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub proof_height: Option<Height>,
}

/// Queries the channel client state via gRPC.
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

    let state = match ChannelClientStateResponse::decode(raw.as_slice()) {
        Ok(s) => s,
        Err(e) => {
            deps.api
                .debug(&format!("WASMDEBUG: failed to decode protobuf: {:?}", e));
            return Err(StdError::generic_err(format!(
                "failed to decode protobuf: {:?}",
                e
            )));
        }
    };

    deps.api.debug(&format!("WASMDEBUG: state: {:?}", state));

    // If we have an IdentifiedClientState, decode its client_state field from the Any wrapper.
    if let Some(ref identified) = state.identified_client_state {
        if let Some(ref any) = identified.client_state {
            if any.type_url == "/ibc.lightclients.tendermint.v1.ClientState" {
                let cs = ClientState::decode(any.value.as_slice()).map_err(|e| {
                    StdError::generic_err(format!(
                        "WASMDEBUG: failed to decode inner ClientState: {:?}",
                        e
                    ))
                })?;
                deps.api
                    .debug(&format!("WASMDEBUG: Decoded ClientState: {:?}", cs));
            } else {
                deps.api.debug(&format!(
                    "WASMDEBUG: Unexpected client_state type_url: {}",
                    any.type_url
                ));
            }
        } else {
            deps.api
                .debug("WASMDEBUG: No client_state found in IdentifiedClientState");
        }
    } else {
        deps.api
            .debug("WASMDEBUG: No identified_client_state found");
    }

    Ok(state)
}

pub fn extract_identified_client_state(
    deps: &Deps<NeutronQuery>,
    state: ChannelClientStateResponse,
) -> StdResult<ClientState> {
    if let Some(ref identified) = state.identified_client_state {
        if let Some(ref any) = identified.client_state {
            if any.type_url == "/ibc.lightclients.tendermint.v1.ClientState" {
                let cs = ClientState::decode(any.value.as_slice()).map_err(|e| {
                    StdError::generic_err(format!(
                        "WASMDEBUG: failed to decode inner ClientState: {:?}",
                        e
                    ))
                })?;
                deps.api
                    .debug(&format!("WASMDEBUG: Decoded ClientState: {:?}", cs));
                return Ok(cs);
            } else {
                deps.api.debug(&format!(
                    "WASMDEBUG: Unexpected client_state type_url: {}",
                    any.type_url
                ));
                return Err(StdError::generic_err(format!(
                    "Unexpected client_state type_url: {}",
                    any.type_url
                )));
            }
        } else {
            deps.api
                .debug("WASMDEBUG: No client_state found in IdentifiedClientState");
            return Err(StdError::generic_err("No client_state found"));
        }
    }

    deps.api
        .debug("WASMDEBUG: No identified_client_state found");
    Err(StdError::generic_err("No identified_client_state found"))
}
