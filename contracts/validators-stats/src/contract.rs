use bech32::{encode, ToBase32};
use cosmwasm_std::{
    entry_point, to_json_binary, Decimal, Deps, Order, Reply, StdError, SubMsg, SubMsgResult,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse;
use neutron_sdk::bindings::query::QueryRegisteredQueryResultResponse;
use neutron_sdk::interchain_queries::queries::get_raw_interchain_query_result;
use neutron_sdk::interchain_queries::types::KVReconstruct;
use neutron_sdk::interchain_queries::v045::types::{SigningInfo, StakingValidator, Validator};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v045::{
        new_register_staking_validators_query_msg,
        register_queries::new_register_validators_signing_infos_query_msg,
    },
    sudo::msg::SudoMsg,
    NeutronResult,
};
use sha2::{Digest, Sha256};

use crate::state::{
    MissedBlocks, QueryMsg, ValidatorMissedBlocksForPeriod, ValidatorState, CONFIG, MISSED_BLOCKS,
    SIGNING_INFO_QUERY_ID, STATE_MAP, VALCONS_TO_VALOPER, VALIDATOR_PROFILE_QUERY_ID,
};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg},
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
        avg_block_time: msg.avg_block_time,
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
    let validators: StdResult<Vec<_>> = STATE_MAP
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect();

    to_json_binary(&validators?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    // TODO: Add code to remove queries and withdraw funds from the contract
    // TODO: Add update config support
    // TODO: Add block time change support
    match msg {
        ExecuteMsg::RegisterStatsQueries { validators } => register_stats_queries(deps, validators),
    }
}

fn register_stats_queries(
    deps: DepsMut<NeutronQuery>,
    validators: Vec<String>,
) -> NeutronResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let msg = new_register_staking_validators_query_msg(
        config.connection_id.clone(),
        validators,
        config.profile_update_period,
    )?;

    let sub_msg = SubMsg::reply_on_success(msg, VALIDATOR_PROFILE_REPLY_ID);

    Ok(Response::new().add_submessage(sub_msg))
}

#[entry_point]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: SudoMsg,
) -> NeutronResult<Response<NeutronMsg>> {
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
) -> NeutronResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_kv_query_result call: {query_id:?}",
    ));

    let validator_profile_query_id = VALIDATOR_PROFILE_QUERY_ID.may_load(deps.storage)?;

    let signing_info_query_id: Option<u64> = SIGNING_INFO_QUERY_ID.may_load(deps.storage)?;

    deps.api.debug(&format!(
        "WASMDEBUG: sudo_kv_query_result validator_profile_query_id: {:?}, signing_info_query_id: {:?}",
        validator_profile_query_id.clone(), signing_info_query_id.clone()
    ));

    let optional_query_id = Some(query_id);

    let interchain_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;

    if optional_query_id == validator_profile_query_id {
        return validator_info_sudo(deps, _env, interchain_query_result);
    } else if optional_query_id == signing_info_query_id {
        return signing_info_sudo(deps, _env, interchain_query_result);
    } else {
        deps.api.debug(&format!(
            "WASMDEBUG: sudo_kv_query_result query_id: {:?}",
            query_id
        ));
    }

    Ok(Response::default())
}

fn validator_info_sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    interchain_query_result: QueryRegisteredQueryResultResponse,
) -> NeutronResult<Response<NeutronMsg>> {
    let data: StakingValidator =
        KVReconstruct::reconstruct(&interchain_query_result.result.kv_results)?;

    deps.api
        .debug(&format!("WASMDEBUG: validator_info_sudo data: {data:?}",));

    let signing_info_query_id = SIGNING_INFO_QUERY_ID.may_load(deps.storage)?;

    if signing_info_query_id.is_none() {
        return register_signing_infos_query(deps, data.validators);
    }

    for validator in data.validators.iter() {
        let mut validator_state = STATE_MAP
            .may_load(deps.storage, validator.operator_address.clone())?
            .unwrap_or_else(|| ValidatorState {
                valoper_address: validator.operator_address.clone(),
                valcons_address: "".to_string(),
                last_processed_local_height: None,
                last_processed_remote_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Decimal::zero(),
                tombstone: false,
                prev_jailed_state: false,
                jailed_number: Some(0),
            });

        validator_state.last_processed_local_height = Some(env.block.height);
        validator_state.last_processed_remote_height = Some(interchain_query_result.result.height);

        validator_state.last_validated_height = if validator.status == 3 {
            Some(env.block.height)
        } else {
            validator_state.last_validated_height
        };

        validator_state.last_commission_in_range = if let Some(rate) = validator.rate {
            if commission_in_range(rate, Decimal::percent(1), Decimal::percent(10)) {
                Some(env.block.height)
            } else {
                validator_state.last_commission_in_range
            }
        } else {
            validator_state.last_commission_in_range
        };

        validator_state.jailed_number = if !validator_state.prev_jailed_state && validator.jailed {
            validator_state.prev_jailed_state = true;
            Some(validator_state.jailed_number.unwrap_or(0) + 1)
        } else if validator_state.prev_jailed_state && !validator.jailed {
            validator_state.prev_jailed_state = false;
            validator_state.jailed_number
        } else {
            validator_state.jailed_number
        };

        STATE_MAP.save(
            deps.storage,
            validator.operator_address.clone(),
            &validator_state,
        )?;
    }

    Ok(Response::new())
}

// TODO: move min/max commission to config
fn commission_in_range(rate: Decimal, min: Decimal, max: Decimal) -> bool {
    rate >= min && rate <= max
}

fn signing_info_sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    interchain_query_result: QueryRegisteredQueryResultResponse,
) -> NeutronResult<Response<NeutronMsg>> {
    let data: SigningInfo = KVReconstruct::reconstruct(&interchain_query_result.result.kv_results)?;
    deps.api
        .debug(&format!("WASMDEBUG: signing_info_sudo data: {data:?}",));

    for info in data.signing_infos.iter() {
        let valoper_address = VALCONS_TO_VALOPER.may_load(deps.storage, info.address.clone())?;

        if valoper_address.is_none() {
            deps.api.debug(&format!(
                "WASMDEBUG: signing_info_sudo: validator operator address was not found: {:?}",
                info.address.clone()
            ));
            continue;
        }

        let mut all_missed_blocks = MISSED_BLOCKS.may_load(deps.storage)?.unwrap_or_default();
        if !all_missed_blocks.is_empty()
            && all_missed_blocks[0].timestamp <= env.block.time.seconds() - 60 * 60 * 24 * 30
        // TODO: move 30 days to config
        {
            all_missed_blocks.remove(0);
        }

        deps.api.debug("WASMDEBUG: signing_info_sudo: 1");

        let mut missed_blocks = MissedBlocks {
            remote_height: interchain_query_result.result.height,
            timestamp: env.block.time.seconds(),
            validators: Vec::new(),
        };

        deps.api.debug("WASMDEBUG: signing_info_sudo: 2");

        if let Some(address) = valoper_address {
            deps.api.debug(&format!(
                "WASMDEBUG: signing_info_sudo: valoper address: {address:?}"
            ));

            let mut validator_state = STATE_MAP
                .may_load(deps.storage, address.clone())?
                .unwrap_or_else(|| ValidatorState {
                    valoper_address: address.clone(),
                    valcons_address: info.address.clone(),
                    last_processed_local_height: None,
                    last_processed_remote_height: None,
                    last_validated_height: None,
                    last_commission_in_range: None,
                    uptime: Decimal::zero(),
                    tombstone: false,
                    prev_jailed_state: false,
                    jailed_number: Some(0),
                });

            deps.api.debug("WASMDEBUG: signing_info_sudo: 3");

            validator_state.tombstone = if info.tombstoned {
                true
            } else {
                validator_state.tombstone
            };

            let validator_missed_blocks = ValidatorMissedBlocksForPeriod {
                address: address.clone(),
                missed_blocks: info.missed_blocks_counter as u64,
            };

            missed_blocks.validators.push(validator_missed_blocks);

            let total_blocks = missed_blocks.remote_height
                - (if !all_missed_blocks.is_empty() {
                    all_missed_blocks[0].remote_height
                } else {
                    missed_blocks.remote_height
                });

            deps.api.debug("WASMDEBUG: signing_info_sudo: 4");

            let sum_missed_blocks: u64 = all_missed_blocks
                .iter()
                .flat_map(|x| &x.validators)
                .filter(|x| x.address == address)
                .map(|inner| inner.missed_blocks)
                .sum();

            deps.api.debug("WASMDEBUG: signing_info_sudo: 5");

            let sum_missed_blocks = sum_missed_blocks + info.missed_blocks_counter as u64;
            deps.api.debug("WASMDEBUG: signing_info_sudo: 5.1");
            let missed_blocks_percent = if total_blocks == 0 {
                Decimal::zero()
            } else {
                Decimal::from_ratio(sum_missed_blocks, total_blocks)
            };
            deps.api.debug("WASMDEBUG: signing_info_sudo: 5.2");
            validator_state.uptime = Decimal::one() - missed_blocks_percent;

            deps.api.debug("WASMDEBUG: signing_info_sudo: 6");

            STATE_MAP.save(deps.storage, address.clone(), &validator_state)?;

            deps.api.debug("WASMDEBUG: signing_info_sudo: 7");
        }

        all_missed_blocks.push(missed_blocks);

        deps.api.debug(&format!(
            "WASMDEBUG: signing_info_sudo: all_missed_blocks: {all_missed_blocks:?}",
        ));

        MISSED_BLOCKS.save(deps.storage, &all_missed_blocks)?;
    }

    Ok(Response::new())
}

fn register_signing_infos_query(
    deps: DepsMut<NeutronQuery>,
    validators: Vec<Validator>,
) -> NeutronResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let mut valcons_addresses = Vec::with_capacity(validators.len());

    for validator in validators.iter() {
        if let Some(pubkey) = validator.clone().consensus_pubkey {
            let valcons_address = pubkey_to_address(pubkey, "cosmosvalcons")?;
            deps.api.debug(&format!(
                "WASMDEBUG: validator_info_sudo valcons address: {valcons_address:?}",
            ));

            VALCONS_TO_VALOPER.save(
                deps.storage,
                valcons_address.clone(),
                &validator.operator_address,
            )?;

            valcons_addresses.push(valcons_address);
        }
    }

    let msg = new_register_validators_signing_infos_query_msg(
        config.connection_id,
        valcons_addresses,
        config.info_update_period,
    )?;
    let sub_msg = SubMsg::reply_on_success(msg, SIGNING_INFO_REPLY_ID);

    Ok(Response::new().add_submessage(sub_msg))
}

fn pubkey_to_address(pubkey: Vec<u8>, prefix: &str) -> StdResult<String> {
    if pubkey.len() < 34 {
        return Err(StdError::generic_err("Invalid public key length"));
    }

    let pubkey_bytes = &pubkey[2..];

    let hash = Sha256::digest(pubkey_bytes);
    let addr_bytes = &hash[..20];

    let bech32_addr = encode(prefix, addr_bytes.to_base32(), bech32::Variant::Bech32)
        .map_err(|e| StdError::generic_err(format!("failed to encode to bech32: {e:?}")))?;

    Ok(bech32_addr)
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

    VALIDATOR_PROFILE_QUERY_ID.save(deps.storage, &query_id)?;

    Ok(Response::new())
}

fn signing_info_reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(&format!("WASMDEBUG: signing_info_reply call: {msg:?}",));

    let query_id = get_query_id(msg.result)?;

    deps.api.debug(&format!(
        "WASMDEBUG: signing_info_reply query id: {query_id:?}"
    ));

    SIGNING_INFO_QUERY_ID.save(deps.storage, &query_id)?;

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
