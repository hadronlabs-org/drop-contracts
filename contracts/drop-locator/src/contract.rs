use crate::{
    error::ContractResult,
    msg::{AddChainInfoResponse, ChainInfoReponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, CONFIG, STATE},
};
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
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
        QueryMsg::ChainsInfo {} => query_chains_info(deps),
        QueryMsg::ChainInfo { name } => query_chain_info(deps, name),
    }
}

pub fn query_chain_info(deps: Deps<NeutronQuery>, name: String) -> StdResult<Binary> {
    let chain_info = STATE.load(deps.storage, name.clone())?;
    to_json_binary(&ChainInfoReponse {
        name: name.clone(),
        chain_info: chain_info.clone(),
    })
}

pub fn query_chains_info(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let chains: StdResult<Vec<_>> = STATE
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            item.map(|(key, value)| ChainInfoReponse {
                name: String::from_utf8(key).unwrap(),
                chain_info: value.clone(),
            })
        })
        .collect();

    let chains = chains.unwrap_or_default();

    to_json_binary(&chains)
}

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
    msg: AddChainInfoResponse,
) -> ContractResult<Response<NeutronMsg>> {
    STATE.save(deps.storage, msg.name, &msg.chain_info)?;
    Ok(response(
        "execute-add-chain-info",
        CONTRACT_NAME,
        vec![attr("action", "add-chain-info-call")],
    ))
}
