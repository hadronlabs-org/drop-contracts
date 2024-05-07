use crate::{
    error::ContractResult,
    msg::{AddChainInfo, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, CONFIG, STATE},
};
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use drop_helpers::answer::response;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    CONFIG.save(deps.storage, &Config {})?;
    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        vec![attr("owner", &info.sender)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ChainInfo { chain } => query_chain_info(deps, chain),
    }
}

pub fn query_chain_info(deps: Deps<NeutronQuery>, chain: String) -> StdResult<Binary> {
    let chain_info = STATE.load(deps.storage, chain)?;
    to_json_binary(&chain_info)
}

// pub fn query_chains_info(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
//     let chains: StdResult<Vec<_>> = STATE
//         .range_raw(deps.storage, None, None, Order::Ascending)
//         .map(|item| item.map(|(_key, value)| value))
//         .collect();

//     let chains = chains.unwrap_or_default();

//     to_json_binary(&chains)
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::AddChainInfo(msg) => execute_add_chain_info(deps, env, info, msg),
    }
}

pub fn execute_add_chain_info(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: AddChainInfo,
) -> ContractResult<Response<NeutronMsg>> {
    STATE.save(deps.storage, msg.key, &msg.chain_info)?;
    Ok(response(
        "execute-add-chain-info",
        CONTRACT_NAME,
        vec![attr("action", "add-chain-info-call")],
    ))
}
