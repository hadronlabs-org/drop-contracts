use prost::Message;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_json, to_json_vec, Binary, Deps, QueryRequest, StdError, StdResult, Uint64,
};
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
// #[derive(Clone, PartialEq, JsonSchema, Serialize, Deserialize, Debug)]
pub struct Height {
    pub revision_number: Uint64,
    pub revision_height: Uint64,
}

#[cw_serde]
pub struct CustomAny {
    #[serde(rename = "@type")]
    pub any_type: String,
    pub type_url: String,
    pub value: Vec<u8>,
    // pub chain_id: String,
}

#[cw_serde]
pub struct IdentifiedClientState {
    pub client_id: String,
    pub client_state: ClientState,
}
#[cw_serde]
// #[derive(Clone, PartialEq, JsonSchema, Serialize, Deserialize, Debug)]
pub struct ChannelClientStateResponse {
    pub identified_client_state: Option<IdentifiedClientState>,
    pub proof: Option<Vec<u8>>,
    pub proof_height: Height,
}

#[test]
fn test() {
    let x = Binary::from_base64("eyJpZGVudGlmaWVkX2NsaWVudF9zdGF0ZSI6eyJjbGllbnRfaWQiOiIwNy10ZW5kZXJtaW50LTEiLCJjbGllbnRfc3RhdGUiOnsiQHR5cGUiOiIvaWJjLmxpZ2h0Y2xpZW50cy50ZW5kZXJtaW50LnYxLkNsaWVudFN0YXRlIiwiY2hhaW5faWQiOiJ0ZXN0Z2FpYSIsInRydXN0X2xldmVsIjp7Im51bWVyYXRvciI6IjEiLCJkZW5vbWluYXRvciI6IjMifSwidHJ1c3RpbmdfcGVyaW9kIjoiMTIwcyIsInVuYm9uZGluZ19wZXJpb2QiOiIzNjBzIiwibWF4X2Nsb2NrX2RyaWZ0IjoiNjVzIiwiZnJvemVuX2hlaWdodCI6eyJyZXZpc2lvbl9udW1iZXIiOiIwIiwicmV2aXNpb25faGVpZ2h0IjoiMCJ9LCJsYXRlc3RfaGVpZ2h0Ijp7InJldmlzaW9uX251bWJlciI6IjAiLCJyZXZpc2lvbl9oZWlnaHQiOiIzNyJ9LCJwcm9vZl9zcGVjcyI6W3sibGVhZl9zcGVjIjp7Imhhc2giOiJTSEEyNTYiLCJwcmVoYXNoX2tleSI6Ik5PX0hBU0giLCJwcmVoYXNoX3ZhbHVlIjoiU0hBMjU2IiwibGVuZ3RoIjoiVkFSX1BST1RPIiwicHJlZml4IjoiQUE9PSJ9LCJpbm5lcl9zcGVjIjp7ImNoaWxkX29yZGVyIjpbMCwxXSwiY2hpbGRfc2l6ZSI6MzMsIm1pbl9wcmVmaXhfbGVuZ3RoIjo0LCJtYXhfcHJlZml4X2xlbmd0aCI6MTIsImVtcHR5X2NoaWxkIjpudWxsLCJoYXNoIjoiU0hBMjU2In0sIm1heF9kZXB0aCI6MCwibWluX2RlcHRoIjowLCJwcmVoYXNoX2tleV9iZWZvcmVfY29tcGFyaXNvbiI6ZmFsc2V9LHsibGVhZl9zcGVjIjp7Imhhc2giOiJTSEEyNTYiLCJwcmVoYXNoX2tleSI6Ik5PX0hBU0giLCJwcmVoYXNoX3ZhbHVlIjoiU0hBMjU2IiwibGVuZ3RoIjoiVkFSX1BST1RPIiwicHJlZml4IjoiQUE9PSJ9LCJpbm5lcl9zcGVjIjp7ImNoaWxkX29yZGVyIjpbMCwxXSwiY2hpbGRfc2l6ZSI6MzIsIm1pbl9wcmVmaXhfbGVuZ3RoIjoxLCJtYXhfcHJlZml4X2xlbmd0aCI6MSwiZW1wdHlfY2hpbGQiOm51bGwsImhhc2giOiJTSEEyNTYifSwibWF4X2RlcHRoIjowLCJtaW5fZGVwdGgiOjAsInByZWhhc2hfa2V5X2JlZm9yZV9jb21wYXJpc29uIjpmYWxzZX1dLCJ1cGdyYWRlX3BhdGgiOlsidXBncmFkZSIsInVwZ3JhZGVkSUJDU3RhdGUiXSwiYWxsb3dfdXBkYXRlX2FmdGVyX2V4cGlyeSI6dHJ1ZSwiYWxsb3dfdXBkYXRlX2FmdGVyX21pc2JlaGF2aW91ciI6dHJ1ZX19LCJwcm9vZiI6bnVsbCwicHJvb2ZfaGVpZ2h0Ijp7InJldmlzaW9uX251bWJlciI6IjAiLCJyZXZpc2lvbl9oZWlnaHQiOiIzNTQifX0=").unwrap();
    let y: ChannelClientStateResponse = from_json(&x).unwrap();
    println!("{:?}", y);
}

pub fn query_client_state(
    deps: &Deps<NeutronQuery>,
    channel_id: String,
    port_id: String,
) -> StdResult<ChannelClientStateResponse> {
    let request = &QueryRequest::Stargate {
        path: "/ibc.core.channel.v1.Query/ChannelClientState".to_string(),
        data: cosmos_sdk_proto::ibc::core::channel::v1::QueryChannelClientStateRequest {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
        }
        .encode_to_vec()
        .into(),
    };
    let raw = to_json_vec(request).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {serialize_err}"))
    })?;
    let data = deps.querier.raw_query(&raw);
    deps.api
        .debug(&format!("WASMDEBUG: query_client_state: data: {data:?}"));
    let state = deps.querier
            .query(request)
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Query channel state for channel {channel_id} and port {port_id} failed: {e}, perhaps, this is wrong channel_id/port_id?"
                ))
            });

    deps.api
        .debug(&format!("WASMDEBUG: query_client_state: {state:?}"));

    state
}
