use cosmwasm_std::{
    attr, to_json_binary, Attribute, BankMsg, Coin, CosmosMsg, Decimal, Deps, Uint128,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_staking_base::error::native_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::native_bond_provider::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::native_bond_provider::{Config, ConfigOptional, CONFIG};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let staker_contract = deps.api.addr_validate(&msg.staker_contract)?;
    let config = &Config {
        staker_contract: staker_contract.clone(),
        base_denom: msg.base_denom.to_string(),
    };
    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("staker_contract", staker_contract),
            attr("base_denom", msg.base_denom),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::CanBond { denom } => query_can_bond(deps, denom),
        QueryMsg::CanProcessOnIdle {} => Ok(to_json_binary(&false)?),
        QueryMsg::TokensAmount {
            coin,
            exchange_rate,
        } => query_token_amount(deps, coin, exchange_rate),
        QueryMsg::AsyncTokensAmount {} => Ok(to_json_binary(&Uint128::zero())?),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_can_bond(deps: Deps<NeutronQuery>, denom: String) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    Ok(to_json_binary(&can_bond(config.base_denom, denom))?)
}

fn query_token_amount(
    deps: Deps<NeutronQuery>,
    coin: Coin,
    exchange_rate: Decimal,
) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    if can_bond(config.base_denom, coin.denom) {
        let issue_amount = coin.amount * (Decimal::one() / exchange_rate);

        return Ok(to_json_binary(&issue_amount)?);
    }

    Err(ContractError::InvalidDenom {})
}

fn can_bond(base_denom: String, denom: String) -> bool {
    base_denom == denom
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::Bond {} => execute_bond(deps, info),
        ExecuteMsg::ProcessOnIdle {} => Err(ContractError::MessageIsNotSupported {}),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut state = CONFIG.load(deps.storage)?;
    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(staker_contract) = new_config.staker_contract {
        state.staker_contract = deps.api.addr_validate(staker_contract.as_ref())?;
        attrs.push(attr("staker_contract", staker_contract))
    }

    if let Some(base_denom) = new_config.base_denom {
        state.base_denom = base_denom.to_string();
        attrs.push(attr("base_denom", base_denom));
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let Coin { amount, denom } = cw_utils::one_coin(&info)?;
    let config = CONFIG.load(deps.storage)?;

    if denom != config.base_denom {
        return Err(ContractError::InvalidDenom {});
    }

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: config.staker_contract.to_string(),
        amount: vec![Coin::new(amount.u128(), denom.to_string())],
    });

    Ok(response(
        "bond",
        CONTRACT_NAME,
        [attr_coin("received_funds", amount.to_string(), denom)],
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
