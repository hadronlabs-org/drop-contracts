use astroport::asset::{Asset, AssetInfo};
use astroport::pair::ExecuteMsg as PairExecuteMsg;
use astroport::router::ExecuteMsg as RouterExecuteMsg;
use astroport::router::SwapOperation;
use cosmwasm_std::{
    attr, ensure_eq, entry_point, to_json_binary, Attribute, Coin, CosmosMsg, Deps, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::{attr_coin, response};
use lido_staking_base::error::astroport_exchange_handler::{ContractError, ContractResult};
use lido_staking_base::msg::astroport_exchange_handler::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use lido_staking_base::state::astroport_exchange_handler::{
    CORE_ADDRESS, CRON_ADDRESS, FROM_DENOM, ROUTER_CONTRACT_ADDRESS,
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let core = deps.api.addr_validate(&msg.core_address)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(core.as_ref()))?;
    CORE_ADDRESS.save(deps.storage, &core)?;

    let cron = deps.api.addr_validate(&msg.cron_address)?;
    CRON_ADDRESS.save(deps.storage, &cron)?;

    let router_contract = deps.api.addr_validate(&msg.router_contract_address)?;
    ROUTER_CONTRACT_ADDRESS.save(deps.storage, &router_contract)?;

    FROM_DENOM.save(deps.storage, &msg.from_denom)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("core_address", msg.core_address),
            attr("cron_address", msg.cron_address),
            attr("router_contract_address", msg.router_contract_address),
            attr("from_denom", msg.from_denom),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();
    let cron_address = CRON_ADDRESS.load(deps.storage)?.into_string();
    let router_contract_address = ROUTER_CONTRACT_ADDRESS.load(deps.storage)?.into_string();
    let from_denom = FROM_DENOM.load(deps.storage)?;

    to_json_binary(&ConfigResponse {
        core_address,
        cron_address,
        router_contract_address,
        from_denom,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            core_address,
            cron_address,
            router_contract_address,
            from_denom,
        } => exec_config_update(
            deps,
            info,
            core_address,
            cron_address,
            router_contract_address,
            from_denom,
        ),
        ExecuteMsg::Exchange { coin } => exec_exchange(deps, info, coin),
        ExecuteMsg::SetRouteAndSwap { operations } => {
            exec_route_and_swap(deps, env, info, operations)
        }
        ExecuteMsg::DirectSwap { contract_address } => {
            exec_direct_swap(deps, env, info, contract_address)
        }
    }
}

fn exec_config_update(
    deps: DepsMut,
    info: MessageInfo,
    core_address: Option<String>,
    cron_address: Option<String>,
    router_contract_address: Option<String>,
    from_denom: Option<String>,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(core_address) = core_address {
        let core_address = deps.api.addr_validate(&core_address)?;
        CORE_ADDRESS.save(deps.storage, &core_address)?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(core_address.as_ref()))?;
        attrs.push(attr("core_address", core_address))
    }

    if let Some(cron_address) = cron_address {
        let cron_address = deps.api.addr_validate(&cron_address)?;
        CRON_ADDRESS.save(deps.storage, &cron_address)?;
        attrs.push(attr("cron_address", cron_address))
    }

    if let Some(router_contract_address) = router_contract_address {
        let router_contract = deps.api.addr_validate(&router_contract_address)?;
        ROUTER_CONTRACT_ADDRESS.save(deps.storage, &router_contract)?;
        attrs.push(attr("router_contract", router_contract_address))
    }

    if let Some(from_denom) = from_denom {
        FROM_DENOM.save(deps.storage, &from_denom)?;
        attrs.push(attr("from_denom", from_denom))
    }

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn exec_exchange(deps: DepsMut, info: MessageInfo, coin: Coin) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    Ok(response(
        "exchange",
        CONTRACT_NAME,
        [attr_coin("exchange_coin", coin.amount, coin.denom)],
    ))
}

fn exec_route_and_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operations: Vec<SwapOperation>,
) -> ContractResult<Response> {
    let cron_address = CRON_ADDRESS.load(deps.storage)?.into_string();
    ensure_eq!(cron_address, info.sender, ContractError::Unauthorized {});

    let router_contract_address = ROUTER_CONTRACT_ADDRESS.load(deps.storage)?.into_string();
    let from_denom = FROM_DENOM.load(deps.storage)?;

    let balance = deps
        .querier
        .query_balance(env.contract.address, from_denom.clone())?;

    let exchange_rewards_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: router_contract_address.clone(),
        msg: to_json_binary(&RouterExecuteMsg::ExecuteSwapOperations {
            operations,
            minimum_receive: None,
            to: None,
            max_spread: None,
        })?,
        funds: vec![balance.clone()],
    });

    Ok(response(
        "set_route_and_swap",
        CONTRACT_NAME,
        [
            attr("router_contract", router_contract_address),
            attr_coin("swap_amount", balance.amount, balance.denom),
        ],
    )
    .add_message(exchange_rewards_msg))
}

fn exec_direct_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_address: String,
) -> ContractResult<Response> {
    let cron_address = CRON_ADDRESS.load(deps.storage)?.into_string();
    ensure_eq!(cron_address, info.sender, ContractError::Unauthorized {});

    let from_denom = FROM_DENOM.load(deps.storage)?;
    let contract_addr = deps.api.addr_validate(&contract_address)?;

    let balance = deps
        .querier
        .query_balance(env.contract.address, from_denom.clone())?;

    let exchange_rewards_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_json_binary(&PairExecuteMsg::Swap {
            offer_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: from_denom.clone(),
                },
                amount: balance.amount,
            },
            ask_asset_info: None,
            belief_price: None,
            max_spread: None,
            to: None,
        })?,
        funds: vec![balance.clone()],
    });

    Ok(response(
        "direct_swap",
        CONTRACT_NAME,
        [
            attr("swap_contract", contract_address),
            attr_coin("swap_amount", balance.amount, balance.denom),
        ],
    )
    .add_message(exchange_rewards_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
