use crate::{
    error::ContractResult,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{BASE_DENOM, DENOM},
};
use cosmwasm_std::{
    to_json_binary, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use drop_helpers::answer::response;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            Ok(to_json_binary(&ownership)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::Bond { receiver } => execute_bond(deps, env, info, receiver),
    }
}

fn execute_bond(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let amount = cw_utils::may_pay(&info, BASE_DENOM)?;
    let receiver = receiver
        .map(|a| deps.api.addr_validate(&a))
        .unwrap_or_else(|| Ok(info.sender))?;
    let dntrn_denom = DENOM.load(deps.storage)?;
    let msg = NeutronMsg::submit_mint_tokens(dntrn_denom, amount, receiver);
    Ok(Response::new()
        .add_attribute("action", "bond")
        .add_attribute("amount", amount.to_string())
        .add_message(msg))
}
