use cosmwasm_std::{
    attr, entry_point, to_json_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};
use lido_helpers::answer::response;
use lido_staking_base::{
    msg::hook_tester::{ExecuteMsg, InstantiateMsg},
    state::hook_tester::{Config, CONFIG},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

use crate::error::ContractResult;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("action", "instantiate"),
        attr("puppeteer_addr", &msg.puppeteer_addr),
    ];
    CONFIG.save(
        deps.storage,
        &Config {
            puppeteer_addr: msg.puppeteer_addr,
        },
    )?;
    Ok(response("instantiate", "hook-tester", attrs))
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Delegate {
            validator,
            amount,
            timeout,
        } => execute_delegate(deps, env, validator, amount, timeout),
        ExecuteMsg::Undelegate {
            validator,
            amount,
            timeout,
        } => execute_undelegate(deps, env, validator, amount, timeout),
        ExecuteMsg::Redelegate {
            validator_from,
            validator_to,
            amount,
            timeout,
        } => execute_redelegate(deps, env, validator_from, validator_to, amount, timeout),
        ExecuteMsg::TokenizeShare {
            validator,
            amount,
            timeout,
        } => execute_tokenize_share(deps, env, validator, amount, timeout),
        ExecuteMsg::RedeemShare {
            validator,
            amount,
            denom,
            timeout,
        } => execute_redeem_share(deps, env, validator, amount, denom, timeout),
    }
}

fn execute_delegate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("action", "delegate"),
        attr("validator", validator.clone()),
        attr("amount", amount.to_string()),
    ];
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_addr,
        msg: to_json_binary(&lido_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
            validator,
            amount,
            timeout,
            reply_to: env.contract.address.to_string(),
        })?,
        funds: vec![],
    });
    Ok(response("execute-delegate", "hook-tester", attrs).add_message(msg))
}

fn execute_undelegate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("action", "undelegate"),
        attr("validator", validator.clone()),
        attr("amount", amount.to_string()),
    ];
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_addr,
        msg: to_json_binary(&lido_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
            validator,
            amount,
            timeout,
            reply_to: env.contract.address.to_string(),
        })?,
        funds: vec![],
    });
    Ok(response("execute-undelegate", "hook-tester", attrs).add_message(msg))
}

fn execute_redelegate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    validator_from: String,
    validator_to: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("action", "redelegate"),
        attr("validator_from", validator_from.clone()),
        attr("validator_to", validator_to.clone()),
        attr("amount", amount.to_string()),
    ];
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_addr,
        msg: to_json_binary(&lido_staking_base::msg::puppeteer::ExecuteMsg::Redelegate {
            validator_from,
            validator_to,
            amount,
            timeout,
            reply_to: env.contract.address.to_string(),
        })?,
        funds: vec![],
    });
    Ok(response("execute-redelegate", "hook-tester", attrs).add_message(msg))
}

fn execute_tokenize_share(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    validator: String,
    amount: Uint128,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("action", "tokenize_share"),
        attr("validator", validator.clone()),
        attr("amount", amount.to_string()),
    ];
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_addr,
        msg: to_json_binary(
            &lido_staking_base::msg::puppeteer::ExecuteMsg::TokenizeShare {
                validator,
                amount,
                timeout,
                reply_to: env.contract.address.to_string(),
            },
        )?,
        funds: vec![],
    });
    Ok(response("execute-tokenize-share", "hook-tester", attrs).add_message(msg))
}

fn execute_redeem_share(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    validator: String,
    amount: Uint128,
    denom: String,
    timeout: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("action", "redeem_share"),
        attr("validator", validator.clone()),
        attr("amount", amount.to_string()),
        attr("denom", denom.clone()),
    ];
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_addr,
        msg: to_json_binary(
            &lido_staking_base::msg::puppeteer::ExecuteMsg::RedeemShare {
                validator,
                amount,
                denom,
                timeout,
                reply_to: env.contract.address.to_string(),
            },
        )?,
        funds: vec![],
    });
    Ok(response("execute-redeem-share", "hook-tester", attrs).add_message(msg))
}
