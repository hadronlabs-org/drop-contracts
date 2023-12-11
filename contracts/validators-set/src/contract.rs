use cosmwasm_std::{entry_point, to_json_binary, Addr, Deps, Order};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::NeutronResult;

use crate::state::{QueryMsg, ValidatorInfo, CONFIG, VALIDATORS_SET};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg},
    state::Config,
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-validators_set__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let config = &Config {
        owner: msg.owner,
        stats_contract: msg.stats_contract,
    };

    CONFIG.save(deps.storage, config)?;

    Ok(Response::default())
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
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            stats_contract,
        } => execute_update_config(deps, info, owner, stats_contract),
        ExecuteMsg::UpdateValidators { validators } => execute_update_validators(deps, validators),
        ExecuteMsg::UpdateValidator { validator } => execute_update_validator(deps, validator),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    owner: Option<Addr>,
    stats_contract: Option<Addr>,
) -> NeutronResult<Response<NeutronMsg>> {
    cw_ownable::is_owner(deps.storage, &info.sender)?;

    let mut state = CONFIG.load(deps.storage)?;

    if owner.is_some() && owner != Some(state.clone().owner) {
        state.owner = owner.unwrap_or(state.owner);
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(state.owner.as_ref()))?;
    }

    if stats_contract.is_some() && stats_contract != Some(state.clone().stats_contract) {
        state.stats_contract = stats_contract.unwrap_or(state.stats_contract);
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(Response::default())
}

fn execute_update_validator(
    deps: DepsMut<NeutronQuery>,
    validator: ValidatorInfo,
) -> NeutronResult<Response<NeutronMsg>> {
    // TODO: implement notification of the validator stats contract about new validator
    let valoper_address = validator.valoper_address.clone();

    VALIDATORS_SET.save(deps.storage, valoper_address, &validator)?;

    Ok(Response::default())
}

fn execute_update_validators(
    deps: DepsMut<NeutronQuery>,
    validators: Vec<ValidatorInfo>,
) -> NeutronResult<Response<NeutronMsg>> {
    // TODO: implement notification of the validator stats contract about new validators set
    VALIDATORS_SET.clear(deps.storage);

    for validator in validators {
        let valoper_address = validator.valoper_address.clone();

        VALIDATORS_SET.save(deps.storage, valoper_address, &validator)?;
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

// TODO: add tests
