use cosmwasm_std::{entry_point, to_json_binary, Deps, Reply, StdError, SubMsg, SubMsgResult};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse;
use neutron_sdk::interchain_queries::query_kv_result;
use neutron_sdk::interchain_queries::v045::types::{SigningInfo, StakingValidator};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v045::{
        new_register_staking_validators_query_msg,
        register_queries::new_register_validators_signing_infos_query_msg,
    },
    sudo::msg::SudoMsg,
    NeutronResult,
};

use crate::state::{QueryMsg, CONFIG, SIGNING_INFO_QUERY_ID, STATE, VALIDATOR_PROFILE_QUERY_ID};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, Validator},
    state::{Config, SIGNING_INFO_REPLY_ID, VALIDATOR_PROFILE_REPLY_ID},
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-validators_stats__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;

    let config = &Config {
        connection_id: msg.connection_id,
        port_id: msg.port_id,
        profile_update_period: msg.profile_update_period,
        info_update_period: msg.info_update_period,
        owner,
    };

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    CONFIG.save(deps.storage, config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => query_state(deps, env),
        QueryMsg::Config {} => query_config(deps, env),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

fn query_state(deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    to_json_binary(&state)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterStatsQueries { validators } => register_stats_queries(deps, validators),
    }
}

fn register_stats_queries(
    deps: DepsMut<NeutronQuery>,
    validators: Vec<Validator>,
) -> NeutronResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let valoper_address = validators
        .iter()
        .map(|validator| validator.valoper_address.clone())
        .collect::<Vec<_>>();

    let msg = new_register_staking_validators_query_msg(
        config.connection_id.clone(),
        valoper_address,
        config.profile_update_period,
    )?;

    let sub_msg = SubMsg::reply_on_success(msg, VALIDATOR_PROFILE_REPLY_ID);

    let response = Response::new().add_submessage(sub_msg);

    let valcons_address = validators
        .iter()
        .map(|validator| validator.valcons_address.clone())
        .collect::<Vec<_>>();
    let msg = new_register_validators_signing_infos_query_msg(
        config.connection_id,
        valcons_address,
        config.info_update_period,
    )?;

    let sub_msg = SubMsg::reply_on_success(msg, SIGNING_INFO_REPLY_ID);
    let response = response.add_submessage(sub_msg);

    Ok(response)
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?},  block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, env, query_id),
        _ => Ok(Response::default()),
    }
}

pub fn sudo_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
) -> NeutronResult<Response> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_kv_query_result call: {query_id:?}",
    ));

    let validator_profile_query_id: Option<u64> = VALIDATOR_PROFILE_QUERY_ID
        .load(deps.storage)
        .unwrap_or(None);

    let signing_info_query_id: Option<u64> =
        SIGNING_INFO_QUERY_ID.load(deps.storage).unwrap_or(None);

    deps.api.debug(&format!(
        "WASMDEBUG: sudo_kv_query_result validator_profile_query_id: {:?}, signing_info_query_id: {:?}",
        validator_profile_query_id.clone(), signing_info_query_id.clone()
    ));

    let optional_query_id = Some(query_id);

    if optional_query_id == validator_profile_query_id {
        validator_info_sudo(deps, _env, query_id)?;
    } else if optional_query_id == signing_info_query_id {
        signing_info_sudo(deps, _env, query_id)?;
    } else {
        deps.api.debug(&format!(
            "WASMDEBUG: sudo_kv_query_result query_id: {:?}",
            query_id
        ));
    }

    // let data: Delegations = query_kv_result(deps.as_ref(), query_id)?;
    // deps.api.debug(
    //     format!("WASMDEBUG: sudo_kv_query_result received; query_id: {query_id:?} data: {data:?}")
    //         .as_str(),
    // );
    // let height = env.block.height;
    // let delegations = data.delegations;
    // self.delegations
    //     .save(deps.storage, &(delegations, height))?;

    Ok(Response::default())
}

fn validator_info_sudo(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
) -> NeutronResult<Response> {
    deps.api.debug(&format!(
        "WASMDEBUG: validator_info_sudo query_id: {query_id:?}",
    ));

    let data: StakingValidator = query_kv_result(deps.as_ref(), query_id)?;

    deps.api
        .debug(&format!("WASMDEBUG: validator_info_sudo data: {data:?}",));

    Ok(Response::new())
}

fn signing_info_sudo(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
) -> NeutronResult<Response> {
    deps.api.debug(&format!(
        "WASMDEBUG: signing_info_sudo query_id: {query_id:?}",
    ));

    let data: SigningInfo = query_kv_result(deps.as_ref(), query_id)?;

    deps.api
        .debug(&format!("WASMDEBUG: signing_info_sudo data: {data:?}",));

    Ok(Response::new())
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: reply msg: {msg:?}").as_str());
    match msg.id {
        VALIDATOR_PROFILE_REPLY_ID => validator_info_reply(deps, env, msg),
        SIGNING_INFO_REPLY_ID => signing_info_reply(deps, env, msg),
        _ => Err(StdError::generic_err(format!(
            "unsupported reply message id {}",
            msg.id
        ))),
    }
}

fn validator_info_reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(&format!("WASMDEBUG: validator_info_reply call: {msg:?}",));

    let query_id = get_query_id(msg.result)?;

    deps.api.debug(&format!(
        "WASMDEBUG: validator_info_reply query id: {query_id:?}"
    ));

    VALIDATOR_PROFILE_QUERY_ID.save(deps.storage, &Some(query_id))?;

    Ok(Response::new())
}

fn signing_info_reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(&format!("WASMDEBUG: signing_info_reply call: {msg:?}",));

    let query_id = get_query_id(msg.result)?;

    deps.api.debug(&format!(
        "WASMDEBUG: signing_info_reply query id: {query_id:?}"
    ));

    SIGNING_INFO_QUERY_ID.save(deps.storage, &Some(query_id))?;

    Ok(Response::new())
}

fn get_query_id(msg_result: SubMsgResult) -> StdResult<u64> {
    let res: MsgRegisterInterchainQueryResponse = serde_json_wasm::from_slice(
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
