use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{
    attr, ensure_eq, ensure_ne, entry_point, to_json_binary, Binary, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use lido_staking_base::msg::core::{ExecuteMsg, InstantiateMsg, QueryMsg};
use lido_staking_base::msg::token::ExecuteMsg as TokenExecuteMsg;
use lido_staking_base::state::core::CONFIG;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};
use std::str::FromStr;
const CONTRACT_NAME: &str = concat!("crates.io:lido-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &msg.into())?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::ExchangeRate {} => to_json_binary(&query_exchange_rate(deps, env)?),
    }
}

fn query_exchange_rate(_deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Decimal> {
    Decimal::from_str("1.01")
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond { receiver } => execute_bond(deps, env, info, receiver),
        ExecuteMsg::Unbond { amount } => execute_unbond(deps, env, info, amount),
        ExecuteMsg::UpdateConfig {
            token_contract,
            puppeteer_contract,
            strategy_contract,
            owner,
        } => execute_update_config(
            deps,
            env,
            info,
            token_contract,
            puppeteer_contract,
            strategy_contract,
            owner,
        ),
    }
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let funds = info.funds;
    ensure_ne!(
        funds.len(),
        0,
        ContractError::InvalidFunds {
            reason: "no funds".to_string()
        }
    );
    ensure_eq!(
        funds.len(),
        1,
        ContractError::InvalidFunds {
            reason: "expected 1 denom".to_string()
        }
    );
    let mut attrs = vec![attr("action", "bond")];

    let amount = funds[0].amount;
    let denom = funds[0].denom.to_string();
    check_denom(denom)?;

    let exchange_rate = query_exchange_rate(deps.as_ref(), env)?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));

    let issue_amount = amount * exchange_rate;
    attrs.push(attr("issue_amount", issue_amount.to_string()));

    let receiver = receiver.map_or(Ok::<String, ContractError>(info.sender.to_string()), |a| {
        deps.api.addr_validate(&a)?;
        Ok(a)
    })?;
    attrs.push(attr("receiver", receiver.clone()));

    let msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.token_contract,
        msg: to_json_binary(&TokenExecuteMsg::Mint {
            amount: issue_amount,
            receiver,
        })?,
        funds: vec![],
    })];

    Ok(Response::default().add_messages(msgs).add_attributes(attrs))
}

fn check_denom(_denom: String) -> ContractResult<()> {
    //todo: check denom
    Ok(())
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    token_contract: Option<String>,
    puppeteer_contract: Option<String>,
    strategy_contract: Option<String>,
    owner: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    ensure_eq!(config.owner, info.sender, ContractError::Unauthorized {});

    let mut attrs = vec![attr("action", "update_config")];
    if let Some(token_contract) = token_contract {
        config.token_contract = token_contract.clone();
        attrs.push(attr("token_contract", token_contract));
    }
    if let Some(puppeteer_contract) = puppeteer_contract {
        config.puppeteer_contract = puppeteer_contract.clone();
        attrs.push(attr("puppeteer_contract", puppeteer_contract));
    }
    if let Some(strategy_contract) = strategy_contract {
        config.strategy_contract = strategy_contract.clone();
        attrs.push(attr("strategy_contract", strategy_contract));
    }
    if let Some(owner) = owner {
        config.owner = owner.clone();
        attrs.push(attr("owner", owner));
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default().add_attributes(attrs))
}

fn execute_unbond(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    _amount: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    unimplemented!("todo");
}
