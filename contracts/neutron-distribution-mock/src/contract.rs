use cosmwasm_std::{
    ensure, to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
};
use drop_staking_base::{
    error::neutron_distribution_mock::ContractResult,
    msg::neutron_distribution_mock::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::neutron_distribution_mock::USERS,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let mut response = Response::new();
    match msg {
        ExecuteMsg::SetWithdrawAddress { address } => {
            let address = deps.api.addr_validate(&address)?;
            USERS.update(deps.storage, &info.sender, |user| {
                let mut user = user.unwrap_or_default();
                user.rewards_address = Some(address);
                ContractResult::Ok(user)
            })?;
        }
        ExecuteMsg::WithdrawRewards {} => {
            let mut user = USERS.load(deps.storage, &info.sender)?;
            ensure!(!user.rewards.is_empty(), StdError::not_found("no rewards"));
            let rewards = user.rewards;
            user.rewards = vec![];
            USERS.save(deps.storage, &info.sender, &user)?;
            let msg = BankMsg::Send {
                amount: rewards,
                to_address: user.rewards_address.unwrap_or(info.sender).into_string(),
            };
            response = response.add_message(msg);
        }
    }
    Ok(response)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::PendingRewards { address } => {
            let address = deps.api.addr_validate(&address)?;
            let user = USERS.load(deps.storage, &address)?;
            Ok(to_json_binary(&user.rewards)?)
        }
    }
}
