use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use drop_helpers::answer::response;
use drop_helpers::icq::new_delegations_and_balance_query_msg;
use drop_helpers::query_id::get_query_id;
use drop_staking_base::state::icq_adapter::{Config, ConfigOptional, IcqAdapter};
use drop_staking_base::{
    error::icq_adapter::{ContractError, ContractResult},
    msg::icq_adapter::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use neutron_sdk::interchain_queries::queries::get_raw_interchain_query_result;
use neutron_sdk::sudo::msg::SudoMsg;
use neutron_sdk::NeutronResult;

use std::{env, vec};

use crate::msg::Options;
use crate::store::{
    BalancesAndDelegations, BalancesAndDelegationsState, ResultReconstruct,
    DELEGATIONS_AND_BALANCES, DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK,
    LAST_COMPLETE_DELEGATIONS_AND_BALANCES_KEY,
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg<Options>,
) -> ContractResult<Response<NeutronMsg>> {
    let adapter: IcqAdapter<Options> = IcqAdapter::new();
    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;
    let router = deps.api.addr_validate(&msg.router)?;
    bech32::decode(&msg.ica).map_err(|_| ContractError::InvalidIca)?;
    let attrs = vec![
        attr("action", "instantiate"),
        attr("router", router.to_string()),
        attr("owner", owner),
        attr("ica", msg.ica.to_string()),
    ];
    let config: Config<Options> = Config {
        router,
        remote_denom: msg.remote_denom,
        ica: msg.ica,
        options: msg.options,
    };
    adapter.config.save(deps.storage, &config)?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg<Empty>) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => {
            let adapter: IcqAdapter<Options> = IcqAdapter::new();
            Ok(to_json_binary(&adapter.config.load(deps.storage)?)?)
        }
        QueryMsg::Extention(_) => todo!(),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Options>,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateValidatorSet { validators } => {
            register_delegations_and_balance_query(deps, info, validators)
        }
        ExecuteMsg::UpdateConfig { new_config } => update_config(deps, info, new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            let attrs = vec![attr("action", "update_ownership")];
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response("update_ownership", CONTRACT_NAME, attrs))
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;
    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }
    Ok(Response::new())
}

pub fn update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional<Options>,
) -> ContractResult<Response<NeutronMsg>> {
    let adapter: IcqAdapter<Options> = IcqAdapter::new();
    let config = adapter.config.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut updated_config = Config {
        remote_denom: config.remote_denom,
        router: config.router,
        ica: config.ica,
        options: new_config.options.unwrap_or(config.options),
    };
    let mut attrs = vec![attr("action", "update_config")];
    if let Some(ica) = new_config.ica {
        bech32::decode(&ica).map_err(|_| ContractError::InvalidIca)?;
        attrs.push(attr("ica", ica.clone()));
        updated_config.ica = ica;
    }
    if let Some(router) = new_config.router {
        let router = deps.api.addr_validate(&router)?;
        attrs.push(attr("router", router.to_string()));
        updated_config.router = router;
    }
    if let Some(remote_denom) = new_config.remote_denom {
        attrs.push(attr("remote_denom", remote_denom.clone()));
        updated_config.remote_denom = remote_denom;
    }
    adapter.config.save(deps.storage, &updated_config)?;
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn register_delegations_and_balance_query(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let adapter: IcqAdapter<Options> = IcqAdapter::new();
    let config = adapter.config.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    cosmwasm_std::ensure!(
        validators.len() < u16::MAX as usize,
        StdError::generic_err("Too many validators provided")
    );
    let current_queries: Vec<u64> = DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let messages = current_queries
        .iter()
        .map(|query_id| {
            DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK.remove(deps.storage, *query_id);
            NeutronMsg::remove_interchain_query(*query_id)
        })
        .collect::<Vec<_>>();

    let mut submessages = vec![];

    for (i, chunk) in validators
        .chunks(config.options.delegations_queries_chunk_size as usize)
        .enumerate()
    {
        submessages.push(SubMsg::reply_on_success(
            new_delegations_and_balance_query_msg(
                config.options.connection_id.clone(),
                config.ica.clone(),
                config.remote_denom.clone(),
                chunk.to_vec(),
                config.options.update_period,
                config.options.sdk_version.as_str(),
            )?,
            i as u64,
        ));
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_submessages(submessages))
}

pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    register_delegations_and_balance_query_reply(deps, msg)
}

pub fn register_delegations_and_balance_query_reply(
    deps: DepsMut,
    msg: Reply,
) -> StdResult<Response> {
    let query_id = get_query_id(msg.result)?;
    DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK.save(deps.storage, query_id, &msg.id)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: SudoMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        SudoMsg::KVQueryResult { query_id } => {
            sudo_delegations_and_balance_kv_query_result(deps, env, query_id)
        }
        _ => unreachable!(),
    }
}

fn sudo_delegations_and_balance_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    query_id: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let adapter: IcqAdapter<Options> = IcqAdapter::new();
    let config = adapter.config.load(deps.storage)?;
    let chunks_len = DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    let chunk_id = DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK.load(deps.storage, query_id)?;
    let (remote_height, kv_results) = {
        let registered_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;
        (
            registered_query_result.result.height,
            registered_query_result.result.kv_results,
        )
    };
    let data: BalancesAndDelegations =
        ResultReconstruct::reconstruct(&kv_results, &config.options.sdk_version, None)?;

    let new_state = match DELEGATIONS_AND_BALANCES.may_load(deps.storage, remote_height)? {
        Some(mut state) => {
            if !state.collected_chunks.contains(&chunk_id) {
                state
                    .data
                    .delegations
                    .delegations
                    .extend(data.delegations.delegations);
                state.collected_chunks.push(chunk_id);
            }
            state
        }
        None => BalancesAndDelegationsState {
            data,
            remote_height,
            local_height: env.block.height,
            timestamp: env.block.time,
            collected_chunks: vec![chunk_id],
        },
    };
    if new_state.collected_chunks.len() == chunks_len {
        let prev_key = LAST_COMPLETE_DELEGATIONS_AND_BALANCES_KEY
            .load(deps.storage)
            .unwrap_or_default();
        if prev_key < remote_height {
            LAST_COMPLETE_DELEGATIONS_AND_BALANCES_KEY.save(deps.storage, &remote_height)?;
        }
    }

    DELEGATIONS_AND_BALANCES.save(deps.storage, remote_height, &new_state)?;
    Ok(Response::default())
}
