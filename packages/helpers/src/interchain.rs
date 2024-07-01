use cosmwasm_std::{Binary, StdError};
use neutron_sdk::{bindings::types::ProtobufAny, NeutronError, NeutronResult};

pub fn prepare_any_msg<T: prost::Message>(msg: T, type_url: &str) -> NeutronResult<ProtobufAny> {
    let mut buf = Vec::with_capacity(msg.encoded_len());

    if let Err(e) = msg.encode(&mut buf) {
        return Err(NeutronError::Std(StdError::generic_err(format!(
            "Encode error: {e}"
        ))));
    }
    Ok(ProtobufAny {
        type_url: type_url.to_string(),
        value: Binary::from(buf),
    })
}
