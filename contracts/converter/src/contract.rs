use crate::error::{ContractError, ContractResult};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, RATE};
use cosmwasm_std::{attr, coin, to_json_binary, CosmosMsg, Decimal, Deps};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{assert_owner, get_ownership};
use drop_helpers::answer::response;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    deps.api.addr_validate(&msg.owner)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;
    if msg.from_token == msg.to_token {
        return Err(ContractError::SameTokens {});
    }
    CONFIG.save(
        deps.storage,
        &Config {
            from_token: msg.from_token.clone(),
            to_token: msg.to_token.clone(),
        },
    )?;
    RATE.save(deps.storage, &msg.rate)?;
    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("owner", msg.owner),
            attr("from_token", msg.from_token),
            attr("to_token", msg.to_token),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::Rate() => query_rate(deps, env),
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
    }
}

fn query_config(deps: Deps, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_rate(deps: Deps, _env: Env) -> ContractResult<Binary> {
    let rate = RATE.load(deps.storage)?;
    Ok(to_json_binary(&rate)?)
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
        ExecuteMsg::UpdateRate { rate } => exec_rate_update(deps, info, rate),
        ExecuteMsg::Swap { receiver } => exec_swap(receiver, deps, env, info),
    }
}

fn exec_rate_update(
    deps: DepsMut,
    info: MessageInfo,
    rate: Decimal,
) -> ContractResult<Response<NeutronMsg>> {
    assert_owner(deps.storage, &info.sender)?;
    RATE.save(deps.storage, &rate)?;
    Ok(response(
        "execute-update-rate",
        CONTRACT_NAME,
        [attr("new_rate", rate.to_string())],
    ))
}

fn exec_swap(
    receiver: String,
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let rate = RATE.load(deps.storage)?;
    deps.api.addr_validate(&receiver)?;

    let from_amount = cw_utils::must_pay(&info, &config.from_token)?;
    let payout_amount = from_amount * rate;
    let send_msg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: receiver,
        amount: vec![coin(payout_amount.u128(), &config.to_token)],
    });

    Ok(response(
        "execute-swap",
        CONTRACT_NAME,
        [
            attr("from", config.from_token),
            attr("to", config.to_token),
            attr("amount_in", from_amount.to_string()),
            attr("amount_out", payout_amount.to_string()),
        ],
    )
    .add_message(send_msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;
    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }
    Ok(Response::new())
}
