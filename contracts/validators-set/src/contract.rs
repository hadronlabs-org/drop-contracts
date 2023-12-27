use cosmwasm_std::{attr, ensure_eq, entry_point, to_json_binary, Addr, Deps, Order};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::response;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;

use crate::error::{ContractError, ContractResult};
use crate::msg::{ValidatorData, ValidatorInfoUpdate};
use crate::state::{ValidatorInfo, CONFIG, VALIDATORS_SET};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::Config,
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let core = deps.api.addr_validate(&msg.core)?;
    let stats_contract = deps.api.addr_validate(&msg.stats_contract)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.core.as_ref()))?;

    let config = &Config {
        core: core.clone(),
        stats_contract: stats_contract.clone(),
    };

    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("core", core), attr("stats_contract", stats_contract)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::Validator { valoper } => query_validator(deps, valoper),
        QueryMsg::Validators {} => query_validators(deps),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

fn query_validator(deps: Deps<NeutronQuery>, valoper: Addr) -> StdResult<Binary> {
    let validators = VALIDATORS_SET.may_load(deps.storage, valoper.to_string())?;

    to_json_binary(&validators)
}

fn query_validators(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let validators: StdResult<Vec<_>> = VALIDATORS_SET
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect();

    to_json_binary(&validators?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateConfig {
            core,
            stats_contract,
        } => execute_update_config(deps, info, core, stats_contract),
        ExecuteMsg::UpdateValidators { validators } => {
            execute_update_validators(deps, info, validators)
        }
        ExecuteMsg::UpdateValidator { validator } => {
            execute_update_validator(deps, info, validator)
        }
        ExecuteMsg::UpdateValidatorInfo { validators } => {
            execute_update_validators_info(deps, info, validators)
        }
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    owner: Option<Addr>,
    stats_contract: Option<Addr>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut state = CONFIG.load(deps.storage)?;

    if let Some(owner) = owner {
        if owner != state.core {
            state.core = owner;
            cw_ownable::initialize_owner(deps.storage, deps.api, Some(state.core.as_ref()))?;
        }
    }

    if let Some(stats_contract) = stats_contract {
        if stats_contract != state.stats_contract {
            state.stats_contract = stats_contract;
        }
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(response(
        "update_config",
        CONTRACT_NAME,
        [
            attr("core", state.core),
            attr("stats_contract", state.stats_contract),
        ],
    ))
}

fn execute_update_validator(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validator: ValidatorData,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    // TODO: implement notification of the validator stats contract about new validator
    let valoper_address = validator.valoper_address.clone();

    VALIDATORS_SET.save(
        deps.storage,
        valoper_address.clone(),
        &ValidatorInfo {
            valoper_address: validator.valoper_address,
            weight: validator.weight,
            last_processed_remote_height: None,
            last_processed_local_height: None,
            last_validated_height: None,
            last_commission_in_range: None,
            uptime: Default::default(),
            tombstone: false,
            jailed_number: None,
        },
    )?;

    Ok(response(
        "update_validator",
        CONTRACT_NAME,
        [
            attr("address", valoper_address),
            attr("weight", validator.weight.to_string()),
        ],
    ))
}

fn execute_update_validators(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<ValidatorData>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let total_count = validators.len();

    // TODO: implement notification of the validator stats contract about new validators set
    VALIDATORS_SET.clear(deps.storage);

    for validator in validators {
        let valoper_address = validator.valoper_address.clone();

        VALIDATORS_SET.save(
            deps.storage,
            valoper_address,
            &ValidatorInfo {
                valoper_address: validator.valoper_address,
                weight: validator.weight,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Default::default(),
                tombstone: false,
                jailed_number: None,
            },
        )?;
    }

    Ok(response(
        "update_validators",
        CONTRACT_NAME,
        [attr("total_count", total_count.to_string())],
    ))
}

fn execute_update_validators_info(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators_update: Vec<ValidatorInfoUpdate>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        config.stats_contract,
        info.sender,
        ContractError::Unauthorized {}
    );

    let total_count = validators_update.len();

    for update in validators_update {
        // TODO: Implement logic to modify validator set based in incoming validator info
        let validator =
            VALIDATORS_SET.may_load(deps.storage, update.valoper_address.to_string())?;
        if validator.is_none() {
            continue;
        }
        let mut validator = validator.unwrap();

        if update.last_commission_in_range.is_some() {
            validator.last_commission_in_range = update.last_commission_in_range;
        }
        if update.last_processed_local_height.is_some() {
            validator.last_processed_local_height = update.last_processed_local_height;
        }
        if update.last_processed_remote_height.is_some() {
            validator.last_processed_remote_height = update.last_processed_remote_height;
        }
        if update.last_validated_height.is_some() {
            validator.last_validated_height = update.last_validated_height;
        }
        if update.jailed_number.is_some() {
            validator.jailed_number = update.jailed_number;
        }

        validator.uptime = update.uptime;
        validator.tombstone = update.tombstone;

        VALIDATORS_SET.save(deps.storage, validator.valoper_address.clone(), &validator)?;
    }

    Ok(response(
        "update_validators_info",
        CONTRACT_NAME,
        [attr("total_count", total_count.to_string())],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
