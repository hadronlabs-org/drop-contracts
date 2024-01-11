use cosmwasm_std::{entry_point, to_json_binary, Deps};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::NeutronResult;

use crate::state::{QueryMsg, CONFIG};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg},
    state::Config,
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
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

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let config = &Config { owner };

    CONFIG.save(deps.storage, config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

// TODO: add tests
