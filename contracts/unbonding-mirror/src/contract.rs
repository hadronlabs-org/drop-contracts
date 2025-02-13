use crate::error::ContractResult;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{attr, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use drop_helpers::answer::response;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use std::env;

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const _LOCAL_DENOM: &str = "untrn";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;
    deps.api.addr_validate(&msg.core_contract)?;
    CONFIG.save(
        deps.storage,
        &Config {
            core_contract: msg.core_contract,
            source_port: msg.source_port,
            source_channel: msg.source_channel,
            ibc_timeout: msg.ibc_timeout,
            prefix: msg.prefix,
            retry_limit: msg.retry_limit,
        },
    )?;
    let attrs = vec![attr("action", "instantiate"), attr("owner", owner)];
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(_deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {}
}
