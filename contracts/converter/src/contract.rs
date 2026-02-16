use crate::error::{ContractError, ContractResult};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, RATE};
use cosmwasm_std::BankMsg;
use cosmwasm_std::{attr, coin, to_json_binary, CosmosMsg, Decimal, Deps};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{assert_owner, get_ownership};
use drop_helpers::answer::response;
use drop_helpers::pause::{pause_guard, set_pause, unpause};
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
            let ownership =
                cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response(
                "execute-update-ownership",
                CONTRACT_NAME,
                ownership.into_attributes(),
            ))
        }
        ExecuteMsg::UpdateRate { rate } => exec_rate_update(deps, info, rate),
        ExecuteMsg::Swap { receiver } => exec_swap(receiver, deps, env, info),
        ExecuteMsg::Clawback { denom } => exec_clawback(deps, env, info, denom),
        ExecuteMsg::Pause {} => exec_pause(deps, info),
        ExecuteMsg::Unpause {} => exec_unpause(deps, info),
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
    pause_guard(deps.storage)?;
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

fn exec_pause(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    use cosmwasm_std::Attribute;
    assert_owner(deps.storage, &info.sender)?;
    set_pause(deps.storage)?;
    Ok(response(
        "execute-pause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn exec_unpause(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    use cosmwasm_std::Attribute;
    assert_owner(deps.storage, &info.sender)?;
    unpause(deps.storage);
    Ok(response(
        "execute-unpause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn exec_clawback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    assert_owner(deps.storage, &info.sender)?;
    let config = CONFIG.load(deps.storage)?;
    let to_address = info.sender.to_string();

    // determine which denom to clawback: provided one or contract's configured `from_token`
    let chosen_denom = denom.unwrap_or(config.from_token.clone());

    let balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), chosen_denom.clone())?
        .amount;

    if balance.is_zero() {
        return Err(ContractError::NoFundsToClawback {});
    }

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: to_address.clone(),
        amount: vec![coin(balance.u128(), &chosen_denom)],
    });

    Ok(response(
        "execute-clawback",
        CONTRACT_NAME,
        [attr("to", to_address), attr("denom", chosen_denom)],
    )
    .add_message(msg))
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
