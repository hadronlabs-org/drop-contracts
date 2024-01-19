use cosmwasm_std::{
    attr, entry_point, to_json_binary, Attribute, Coin, CosmosMsg, Deps, Order, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::{attr_coin, response};
use lido_staking_base::error::astroport_exchange_handler::ContractResult;
use lido_staking_base::msg::astroport_exchange_handler::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use lido_staking_base::state::astroport_exchange_handler::CORE_ADDRESS;

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let core = deps.api.addr_validate(&msg.core_address)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core.as_ref()))?;
    CORE_ADDRESS.save(deps.storage, &core)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("core_address", msg.core_address)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();

    to_json_binary(&ConfigResponse { core_address })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { core_address } => exec_config_update(deps, info, core_address),
        ExecuteMsg::Exchange { coin } => exec_exchange(deps, info, coin),
    }
}

fn exec_config_update(
    deps: DepsMut,
    info: MessageInfo,
    core_address: Option<String>,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        CORE_ADDRESS.save(deps.storage, &core_address)?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(core_address.as_ref()))?;
        attrs.push(attr("core_address", core_address))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn exec_exchange(deps: DepsMut, info: MessageInfo, coin: Coin) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    Ok(response(
        "exchange",
        CONTRACT_NAME,
        [attr_coin("exchange_coin", coin.amount, coin.denom)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
