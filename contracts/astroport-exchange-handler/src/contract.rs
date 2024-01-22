use astroport::asset::{Asset, AssetInfo};
use astroport::pair::ExecuteMsg as PairExecuteMsg;
use astroport::router::ExecuteMsg as RouterExecuteMsg;
use astroport::router::SwapOperation;
use cosmwasm_std::{
    attr, ensure_eq, entry_point, to_json_binary, Attribute, Coin, CosmosMsg, Deps, Uint128,
    WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use lido_helpers::answer::{attr_coin, response};
use lido_staking_base::error::astroport_exchange_handler::{ContractError, ContractResult};
use lido_staking_base::msg::astroport_exchange_handler::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use lido_staking_base::state::astroport_exchange_handler::{Config, CONFIG, SWAP_OPERATIONS};

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
    let owner = deps.api.addr_validate(&msg.owner)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_ref()))?;

    let cron = deps.api.addr_validate(&msg.cron_address)?;
    let core_contract = deps.api.addr_validate(&msg.core_contract)?;
    let swap_contract = deps.api.addr_validate(&msg.swap_contract)?;
    let router_contract = deps.api.addr_validate(&msg.router_contract)?;

    let config = Config {
        owner: msg.owner,
        cron_address: cron.to_string(),
        core_contract: core_contract.to_string(),
        swap_contract: swap_contract.to_string(),
        router_contract: router_contract.to_string(),
        from_denom: msg.from_denom,
        min_rewards: msg.min_rewards,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("owner", msg.owner),
            attr("core_address", msg.core_contract),
            attr("cron_address", msg.cron_address),
            attr("swap_contract", msg.swap_contract),
            attr("router_contract", msg.router_contract),
            attr("from_denom", msg.from_denom),
            attr("min_rewards", msg.min_rewards),
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
    let config = CONFIG.load(deps.storage)?;

    let swap_operations = SWAP_OPERATIONS.may_load(deps.storage)?;

    to_json_binary(&ConfigResponse {
        owner: config.owner,
        core_contract: config.core_contract,
        cron_address: config.cron_address,
        swap_contract: config.swap_contract,
        router_contract: config.router_contract,
        from_denom: config.from_denom,
        min_rewards: config.min_rewards,
        swap_operations,
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
            owner,
            core_contract,
            cron_address,
            router_contract,
            swap_contract,
            from_denom,
            min_rewards,
        } => exec_config_update(
            deps,
            info,
            owner,
            core_contract,
            cron_address,
            router_contract,
            swap_contract,
            from_denom,
            min_rewards,
        ),
        ExecuteMsg::Exchange { coin } => exec_exchange(deps, info, coin),
        ExecuteMsg::UpdateSwapOperations { operations } => {
            exec_update_swap_operations(deps, env, info, operations)
        }
    }
}

fn exec_config_update(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    core_contract: Option<String>,
    cron_address: Option<String>,
    router_contract: Option<String>,
    swap_contract: Option<String>,
    from_denom: Option<String>,
    min_rewards: Option<Uint128>,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();
    if let Some(owner) = owner {
        let owner = deps.api.addr_validate(&owner)?;
        config.owner = owner.to_string();
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_ref()))?;
        attrs.push(attr("owner", owner))
    }

    if let Some(core_contract) = core_contract {
        let core_contract = deps.api.addr_validate(&core_contract)?;
        config.core_contract = core_contract.to_string();
        attrs.push(attr("core_contract", core_contract))
    }

    if let Some(cron_address) = cron_address {
        let cron_address = deps.api.addr_validate(&cron_address)?;
        config.cron_address = cron_address.to_string();
        attrs.push(attr("cron_address", cron_address))
    }

    if let Some(router_contract) = router_contract {
        let router_contract = deps.api.addr_validate(&router_contract)?;
        config.router_contract = router_contract.to_string();
        attrs.push(attr("router_contract", router_contract))
    }

    if let Some(swap_contract) = swap_contract {
        let swap_contract = deps.api.addr_validate(&swap_contract)?;
        config.swap_contract = swap_contract.to_string();
        attrs.push(attr("swap_contract", swap_contract))
    }

    if let Some(from_denom) = from_denom {
        config.from_denom = from_denom.to_string();
        attrs.push(attr("from_denom", from_denom))
    }

    if let Some(min_rewards) = min_rewards {
        config.min_rewards = min_rewards;
        attrs.push(attr("min_rewards", min_rewards))
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

fn exec_update_swap_operations(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operations: Option<Vec<SwapOperation>>,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    if let Some(operations) = operations {
        SWAP_OPERATIONS.save(deps.storage, &operations)?;
        return Ok(response(
            "update_swap_operations",
            CONTRACT_NAME,
            [attr("new_swap_operations", operations.len().to_string())],
        ));
    }

    SWAP_OPERATIONS.remove(deps.storage);
    Ok(response(
        "update_swap_operations",
        CONTRACT_NAME,
        [attr("clear_operations", "1".to_string())],
    ))

    // let cron_address = CRON_ADDRESS.load(deps.storage)?.into_string();
    // ensure_eq!(cron_address, info.sender, ContractError::Unauthorized {});

    // let router_contract_address = ROUTER_CONTRACT_ADDRESS.load(deps.storage)?.into_string();
    // let from_denom = FROM_DENOM.load(deps.storage)?;

    // let balance = deps
    //     .querier
    //     .query_balance(env.contract.address, from_denom.clone())?;

    // let exchange_rewards_msg = CosmosMsg::Wasm(WasmMsg::Execute {
    //     contract_addr: router_contract_address.clone(),
    //     msg: to_json_binary(&RouterExecuteMsg::ExecuteSwapOperations {
    //         operations,
    //         minimum_receive: None,
    //         to: None,
    //         max_spread: None,
    //     })?,
    //     funds: vec![balance.clone()],
    // });

    // Ok(response(
    //     "update_swap_operations",
    //     CONTRACT_NAME,
    //     [
    //         attr("router_contract", router_contract_address),
    //         attr_coin("swap_amount", balance.amount, balance.denom),
    //     ],
    // )
    // .add_message(exchange_rewards_msg))
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
