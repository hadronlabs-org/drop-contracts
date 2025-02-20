use cosmwasm_std::{ensure, BankMsg, DepsMut, Env, MessageInfo, Response, StdError};
use drop_staking_base::{
    error::neutron_distribution_mock::ContractResult,
    msg::neutron_distribution_mock::{ExecuteMsg, InstantiateMsg},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const NTRN_DENOM: &str = "untrn";

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let mut response = Response::new();
    match msg {
        ExecuteMsg::ClaimRewards { to_address } => {
            let user = if let Some(to_address) = to_address {
                deps.api.addr_validate(&to_address)?
            } else {
                info.sender
            };
            let coin = deps
                .querier
                .query_balance(&env.contract.address, NTRN_DENOM)?;
            ensure!(
                !coin.amount.is_zero(),
                StdError::generic_err("balance is zero")
            );

            let msg = BankMsg::Send {
                amount: vec![coin],
                to_address: user.to_string(),
            };
            response = response.add_message(msg);
        }
    }
    Ok(response)
}
