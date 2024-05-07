use crate::{
    error::ContractResult,
    msg::{AddChainList, ChainInfo, ExecuteMsg, InstantiateMsg, QueryMsg, RemoveChainList},
    state::STATE,
};
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult,
};
use cw2::set_contract_version;
use cw_ownable::{get_ownership, update_ownership};
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
    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        vec![attr("owner", &info.sender)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Chains {} => query_chains(deps),
        QueryMsg::Chain { name } => query_chain(deps, name),
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
    }
}

pub fn query_chain(deps: Deps<NeutronQuery>, name: String) -> StdResult<Binary> {
    let chain = STATE.load(deps.storage, name.clone())?;
    to_json_binary(&ChainInfo {
        name: name,
        details: chain,
    })
}

pub fn query_chains(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let chains: StdResult<Vec<_>> = STATE
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            item.map(|(key, value)| ChainInfo {
                name: String::from_utf8(key).unwrap(),
                details: value.clone(),
            })
        })
        .collect();
    let chains = chains?;
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
        ExecuteMsg::AddChains(msg) => execute_add_chains(deps, env, info, msg),
        ExecuteMsg::RemoveChains(msg) => execute_remove_chains(deps, env, info, msg),
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
    }
}

pub fn execute_remove_chains(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: RemoveChainList,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut attrs: Vec<Attribute> = Vec::new();
    msg.names.iter().for_each(|name| {
        STATE.remove(deps.storage, name.clone());
        match STATE.has(deps.storage, name.clone()) {
            true => attrs.push(attr("skip", name.clone())),
            false => attrs.push(attr("remove", name.clone())),
        }
    });
    Ok(response("execute-remove-chains", CONTRACT_NAME, attrs))
}

pub fn execute_add_chains(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AddChainList,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut attrs: Vec<Attribute> = Vec::new();
    for chain in msg.chains {
        match STATE.save(deps.storage, chain.name.clone(), &chain.details.clone()) {
            Ok(_) => attrs.push(attr("add", chain.name.clone())),
            Err(_) => attrs.push(attr("skip", chain.name.clone())),
        }
    }
    Ok(response("execute-add-chains", CONTRACT_NAME, attrs))
}
