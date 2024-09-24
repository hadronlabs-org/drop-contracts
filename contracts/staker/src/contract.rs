use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, CosmosMsg, Deps, Reply, SubMsg, SubMsgResult, Uint128,
    WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use drop_helpers::answer::response;
use drop_puppeteer_base::msg::{IBCTransferReason, ReceiverExecuteMsg};
use drop_staking_base::state::staker::PUPPETEER_TRANSFER_REPLY_ID;
use drop_staking_base::{
    msg::staker::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::staker::{Config, ConfigOptional, CONFIG, NON_STAKED_BALANCE},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronError, NeutronResult,
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LOCAL_DENOM: &str = "untrn";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let attrs = vec![
        attr("contract_name", CONTRACT_NAME),
        attr("contract_version", CONTRACT_VERSION),
        attr("msg", format!("{:?}", msg)),
        attr("sender", &info.sender),
    ];
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.owner.unwrap_or(info.sender.to_string()).as_str()),
    )?;
    let puppeteer_contract = deps.api.addr_validate(msg.puppeteer_contract.as_ref())?;
    let core_contract = deps.api.addr_validate(msg.core_contract.as_ref())?;
    CONFIG.save(
        deps.storage,
        &Config {
            remote_denom: msg.remote_denom,
            base_denom: msg.base_denom,
            puppeteer_contract: puppeteer_contract.into_string(),
            core_contract: core_contract.into_string(),
            min_ibc_transfer: msg.min_ibc_transfer,
        },
    )?;
    NON_STAKED_BALANCE.save(deps.storage, &Uint128::zero())?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::AllBalance {} => query_all_balance(deps, env),
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            to_json_binary(&ownership).map_err(NeutronError::Std)
        }
    }
}

fn query_non_staked_balance(deps: Deps, _env: Env) -> NeutronResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    Ok(to_json_binary(&(balance))?)
}

fn query_all_balance(deps: Deps, env: Env) -> NeutronResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let local_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), config.base_denom)?
        .amount;
    to_json_binary(&(balance + local_balance)).map_err(NeutronError::Std)
}

fn query_config(deps: Deps) -> NeutronResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config).map_err(NeutronError::Std)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, *new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::PuppeteerTransfer {} => execute_puppeteer_transfer(deps, env),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let attrs = vec![
        attr("action", "update_config"),
        attr("new_config", format!("{:?}", new_config)),
    ];
    if let Some(puppeteer_address) = new_config.puppeteer_contract {
        config.puppeteer_contract = puppeteer_address;
    }
    if let Some(min_ibc_transfer) = new_config.min_ibc_transfer {
        config.min_ibc_transfer = min_ibc_transfer;
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_puppeteer_transfer(
    deps: DepsMut<NeutronQuery>,
    env: Env,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let pending_coin = deps
        .querier
        .query_balance(&env.contract.address, config.base_denom)?;
    ensure!(
        pending_coin.amount >= config.min_ibc_transfer,
        ContractError::InvalidFunds {
            reason: "amount is less than min_ibc_transfer".to_string()
        }
    );
    NON_STAKED_BALANCE.update(deps.storage, |balance| {
        StdResult::Ok(balance + pending_coin.amount)
    })?;
    let attrs = vec![
        attr("action", "puppeteer_transfer"),
        attr("pending_amount", pending_coin.amount),
    ];

    let puppeteer_transfer = SubMsg::reply_on_error(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.puppeteer_contract.to_string(),
            msg: to_json_binary(
                &drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
                    reason: IBCTransferReason::Delegate,
                    reply_to: env.contract.address.to_string(),
                },
            )?,
            funds: vec![pending_coin],
        }),
        PUPPETEER_TRANSFER_REPLY_ID,
    );

    Ok(response("puppeteer_transfer", CONTRACT_NAME, attrs).add_submessage(puppeteer_transfer))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );
    if let drop_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) = msg.clone() {
        if let drop_puppeteer_base::msg::Transaction::Stake { items } = success_msg.transaction {
            let amount_to_stake: Uint128 = items.iter().map(|(_, amount)| *amount).sum();

            NON_STAKED_BALANCE.update(deps.storage, |balance| {
                StdResult::Ok(balance - amount_to_stake)
            })?;
        }
    }

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PuppeteerHook(msg))?,
        funds: vec![],
    });

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    )
    .add_message(hook_message))
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

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: Deps, _env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        PUPPETEER_TRANSFER_REPLY_ID => puppeteer_transfer_reply(deps, msg),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn puppeteer_transfer_reply(_deps: Deps, msg: Reply) -> ContractResult<Response> {
    if let SubMsgResult::Err(err) = msg.result {
        return Err(ContractError::PuppeteerError { message: err });
    }

    Ok(Response::new())
}
