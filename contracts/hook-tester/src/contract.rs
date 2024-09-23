use crate::error::ContractResult;
use cosmwasm_std::{
    attr, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, WasmMsg,
};
use drop_helpers::answer::response;
use drop_puppeteer_base::{
    msg::{ResponseHookErrorMsg, ResponseHookMsg, ResponseHookSuccessMsg},
    state::RedeemShareItem,
};
use drop_staking_base::{
    msg::hook_tester::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::hook_tester::{Config, ANSWERS, CONFIG, ERRORS},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> NeutronResult<Response> {
    let attrs = vec![attr("action", "instantiate")];
    ERRORS.save(deps.storage, &vec![])?;
    ANSWERS.save(deps.storage, &vec![])?;
    Ok(response("instantiate", "hook-tester", attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Answers {} => to_json_binary(&ANSWERS.load(deps.storage)?),
        QueryMsg::Errors {} => to_json_binary(&ERRORS.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::SetConfig { puppeteer_addr } => {
            execute_set_config(deps, env, info, puppeteer_addr)
        }
        ExecuteMsg::Undelegate { validator, amount } => {
            execute_undelegate(deps, env, validator, amount)
        }
        ExecuteMsg::Redelegate {
            validator_from,
            validator_to,
            amount,
        } => execute_redelegate(deps, env, validator_from, validator_to, amount),
        ExecuteMsg::TokenizeShare { validator, amount } => {
            execute_tokenize_share(deps, env, validator, amount)
        }
        ExecuteMsg::RedeemShare {
            validator,
            amount,
            denom,
        } => execute_redeem_share(deps, env, validator, amount, denom),
        ExecuteMsg::PuppeteerHook(hook_msg) => match *hook_msg {
            ResponseHookMsg::Success(success_msg) => hook_success(deps, env, info, success_msg),
            ResponseHookMsg::Error(error_msg) => hook_error(deps, env, info, error_msg),
        },
    }
}

fn hook_success(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    answer: ResponseHookSuccessMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "hook-success")];
    ANSWERS.update(deps.storage, |mut answers| -> ContractResult<_> {
        answers.push(answer);
        Ok(answers)
    })?;
    Ok(response("hook-success", "hook-tester", attrs))
}

fn hook_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    answer: ResponseHookErrorMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "hook-success")];
    ERRORS.update(deps.storage, |mut errors| -> ContractResult<_> {
        errors.push(answer);
        Ok(errors)
    })?;
    Ok(response("hook-success", "hook-tester", attrs))
}

fn execute_set_config(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    puppeteer_addr: String,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "set-config")];
    CONFIG.save(deps.storage, &Config { puppeteer_addr })?;
    Ok(response("set-config", "hook-tester", attrs))
}

fn execute_undelegate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    validator: String,
    amount: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("action", "undelegate"),
        attr("validator", validator.clone()),
        attr("amount", amount.to_string()),
    ];
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_addr,
        msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
            items: vec![(validator, amount)],
            batch_id: 0,
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
        msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Redelegate {
            validator_from,
            validator_to,
            amount,
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
            &drop_staking_base::msg::puppeteer::ExecuteMsg::TokenizeShare {
                validator,
                amount,
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
            &drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
                items: vec![RedeemShareItem {
                    remote_denom: denom,
                    amount,
                    local_denom: "some".to_string(),
                }],
                reply_to: env.contract.address.to_string(),
            },
        )?,
        funds: vec![],
    });
    Ok(response("execute-redeem-share", "hook-tester", attrs).add_message(msg))
}
