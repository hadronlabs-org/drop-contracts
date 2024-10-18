use cosmwasm_std::{DepsMut, MessageInfo};
use drop_staking_base::{
    error::withdrawal_exchange::{ContractError, ContractResult},
    msg::withdrawal_exchange::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::withdrawal_exchange::{WITHDRAWAL_TOKEN_ADDRESS},
};
use neutron_sdk::{bindings::{query::NeutronQuery}};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let withdrawal_token = deps.api.addr_validate(&msg.withdrawal_token_address)?;
    WITHDRAWAL_TOKEN_ADDRESS.save(deps.storage, &withdrawal_token)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("withdrawal_token_address", withdrawal_token),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let withdrawal_token_address =
                WITHDRAWAL_TOKEN_ADDRESS.load(deps.storage)?.into_string();
            Ok(to_json_binary(&ConfigResponse {
                withdrawal_token_address,
            })?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Exchange { } => "bla",
    }
}