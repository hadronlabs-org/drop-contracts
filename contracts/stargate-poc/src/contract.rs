use cosmos_sdk_proto::ibc;
use cosmwasm_std::{
    entry_point, to_json_binary, to_json_vec, ContractResult, Deps, Empty, QueryRequest, StdError,
    SystemResult,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use neutron_sdk::NeutronResult;
use prost::Message;
use std::str::from_utf8;

use crate::msg::{InstantiateMsg, MigrateMsg};
use crate::state::QueryMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> NeutronResult<Response> {
    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Trace { hash } => query_trace(deps, env, hash),
    }
}

fn query_trace(deps: Deps, _env: Env, hash: String) -> NeutronResult<Binary> {
    let msg = ibc::applications::transfer::v1::QueryDenomTraceRequest { hash };
    let resp = make_stargate_query(
        deps,
        "/ibc.applications.transfer.v1.Query/DenomTrace".to_string(),
        msg.encode_to_vec(),
    )?;

    Ok(to_json_binary(&resp)?)
}

pub fn make_stargate_query(
    deps: Deps,
    path: String,
    encoded_query_data: Vec<u8>,
) -> StdResult<String> {
    let raw = to_json_vec::<QueryRequest<Empty>>(&QueryRequest::Stargate {
        path,
        data: encoded_query_data.into(),
    })
    .map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;
    match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        // response(value) is base64 encoded bytes
        SystemResult::Ok(ContractResult::Ok(value)) => {
            let str = value.to_base64();
            deps.api
                .debug(format!("WASMDEBUG: make_stargate_query: {:?}", str).as_str());
            from_utf8(value.as_slice())
                .map(|s| s.to_string())
                .map_err(|_e| StdError::generic_err("Unable to encode from utf8"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
