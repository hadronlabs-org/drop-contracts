use crate::{
    error::ContractResult,
    msg::{ExecuteMsg, FactoryInstance, InstantiateMsg, QueryMsg},
    state::{DropInstance, STATE},
};
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::response;
use drop_staking_base::msg::factory::QueryMsg as FactoryQueryMsg;
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
        QueryMsg::FactoryInstance { name } => query_factory_instance(deps, name),
        QueryMsg::FactoryInstances {} => query_factory_instances(deps),
    }
}

pub fn query_factory_instance(deps: Deps<NeutronQuery>, name: String) -> StdResult<Binary> {
    let factory_addr = STATE.load(deps.storage, name)?.factory_addr;
    to_json_binary(&FactoryInstance {
        addr: factory_addr.to_string(),
        contracts: deps
            .querier
            .query_wasm_smart(factory_addr.clone(), &FactoryQueryMsg::State {})?,
    })
}

pub fn query_factory_instances(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let mut drop_instances: Vec<FactoryInstance> = vec![];
    for drop_instance in STATE.range(deps.storage, None, None, Order::Ascending) {
        let factory_addr = drop_instance?.1.factory_addr.clone();
        drop_instances.push(FactoryInstance {
            addr: factory_addr.clone(),
            contracts: deps
                .querier
                .query_wasm_smart(factory_addr.clone(), &FactoryQueryMsg::State {})?,
        })
    }

    to_json_binary(&drop_instances)
}

pub fn query_chain(deps: Deps<NeutronQuery>, name: String) -> StdResult<Binary> {
    let chain = STATE.load(deps.storage, name.clone())?;
    to_json_binary(&DropInstance {
        name,
        factory_addr: chain.factory_addr,
    })
}

pub fn query_chains(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let drop_instances: StdResult<Vec<_>> = STATE
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            item.map(|(key, value)| DropInstance {
                name: key,
                factory_addr: value.factory_addr,
            })
        })
        .collect();
    let drop_instances = drop_instances?;
    to_json_binary(&drop_instances)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::AddChains { chains } => execute_add_chains(deps, env, info, chains),
        ExecuteMsg::RemoveChains { names } => execute_remove_chains(deps, env, info, names),
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
    msg: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    msg.iter().for_each(|name| {
        STATE.remove(deps.storage, name.to_string());
    });
    Ok(response(
        "execute-remove-chains",
        CONTRACT_NAME,
        vec![attr("removed_chains", msg.join(","))],
    ))
}

pub fn execute_add_chains(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Vec<DropInstance>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    for chain in &msg {
        STATE.save(
            deps.storage,
            chain.name.clone(),
            &DropInstance {
                name: chain.name.to_string(),
                factory_addr: chain.factory_addr.clone(),
            },
        )?;
    }
    Ok(response(
        "execute-add-chains",
        CONTRACT_NAME,
        vec![attr(
            "added_chains",
            msg.iter()
                .map(|element| element.name.clone())
                .collect::<Vec<String>>()
                .join(","),
        )],
    ))
}
