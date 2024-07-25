use cosmwasm_std::{entry_point, to_json_binary, BankMsg, Coin, CosmosMsg, Deps, Uint128, WasmMsg};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use drop_staking_base::{
    error::token_distributor::{ContractError, ContractResult},
    msg::token_distributor::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, Token},
    state::token_distributor::{Config, CONFIG},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    CONFIG.save(deps.storage, &msg.config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            to_json_binary(&CONFIG.load(deps.storage)?).map_err(ContractError::from)
        }
        QueryMsg::Ownership {} => query_ownership(deps),
    }
}

pub fn query_ownership(deps: Deps) -> ContractResult<Binary> {
    let ownership = cw_ownable::get_ownership(deps.storage)?;
    to_json_binary(&ownership).map_err(ContractError::from)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Distribute { denoms } => execute_distribute(deps, env, info, denoms),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_config: Config,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::default())
}

pub fn execute_distribute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    denoms: Vec<Token>,
) -> ContractResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let total_share = config
        .receivers
        .iter()
        .fold(Uint128::zero(), |acc, (_, share)| acc + share);
    if total_share.is_zero() {
        return Err(ContractError::NoShares {});
    }
    let mut messages = vec![];
    for denom in denoms {
        match denom {
            Token::Native { denom } => {
                let balance = deps
                    .querier
                    .query_balance(&env.clone().contract.address.to_string(), denom.to_string())?;
                let amount = balance.amount;
                if amount.is_zero() {
                    return Err(ContractError::InsufficientFunds {});
                }
                for (receiver, amount) in
                    recepients_to_amounts(config.receivers.clone(), total_share, amount)
                {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: receiver.to_string(),
                        amount: vec![Coin::new(amount.u128(), denom.to_string())],
                    }));
                }
            }
            Token::CW20 { address } => {
                let balance: cw20::BalanceResponse = deps.querier.query_wasm_smart(
                    address.to_string(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: env.clone().contract.address.into(),
                    },
                )?;
                let amount = balance.balance;
                if amount.is_zero() {
                    return Err(ContractError::InsufficientFunds {});
                }

                for (receiver, sum) in
                    recepients_to_amounts(config.receivers.clone(), total_share, amount)
                {
                    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: address.to_string(),
                        msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                            recipient: receiver,
                            amount: sum,
                        })?,
                        funds: vec![],
                    }));
                }
            }
        }
    }

    Ok(Response::default().add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}

fn recepients_to_amounts(
    recepients: Vec<(String, Uint128)>,
    total: Uint128,
    to_distribute: Uint128,
) -> Vec<(String, Uint128)> {
    let mut amounts = vec![];
    for (addr, share) in recepients {
        let amount = to_distribute * share / total;
        amounts.push((addr, amount));
    }
    amounts
}
