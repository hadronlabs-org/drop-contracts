use astroport::asset::{Asset, AssetInfo};
use astroport::pair::ExecuteMsg as PairExecuteMsg;
use astroport::router::ExecuteMsg as RouterExecuteMsg;
use astroport::router::SwapOperation;
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Attribute, CosmosMsg, Deps, Uint128, WasmMsg,
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
    let pair_contract = deps.api.addr_validate(&msg.pair_contract)?;
    let router_contract = deps.api.addr_validate(&msg.router_contract)?;

    let config = Config {
        owner: msg.owner.clone(),
        cron_address: cron.to_string(),
        core_contract: core_contract.to_string(),
        pair_contract: pair_contract.to_string(),
        router_contract: router_contract.to_string(),
        from_denom: msg.from_denom.clone(),
        min_rewards: msg.min_rewards,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("owner", msg.owner),
            attr("core_contract", msg.core_contract),
            attr("cron_address", msg.cron_address),
            attr("pair_contract", msg.pair_contract),
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
        pair_contract: config.pair_contract,
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
            pair_contract,
            from_denom,
            min_rewards,
        } => exec_config_update(
            deps,
            info,
            owner,
            core_contract,
            cron_address,
            router_contract,
            pair_contract,
            from_denom,
            min_rewards,
        ),
        ExecuteMsg::Exchange {} => exec_exchange(deps, env),
        ExecuteMsg::UpdateSwapOperations { operations } => {
            exec_update_swap_operations(deps, info, operations)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn exec_config_update(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    core_contract: Option<String>,
    cron_address: Option<String>,
    router_contract: Option<String>,
    pair_contract: Option<String>,
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

    if let Some(pair_contract) = pair_contract {
        let pair_contract = deps.api.addr_validate(&pair_contract)?;
        config.pair_contract = pair_contract.to_string();
        attrs.push(attr("pair_contract", pair_contract))
    }

    if let Some(from_denom) = from_denom {
        config.from_denom = from_denom.to_string();
        attrs.push(attr("from_denom", from_denom))
    }

    if let Some(min_rewards) = min_rewards {
        config.min_rewards = min_rewards;
        attrs.push(attr("min_rewards", min_rewards))
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn exec_exchange(deps: DepsMut, env: Env) -> ContractResult<Response> {
    let swap_operations = SWAP_OPERATIONS.may_load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    let from_denom = config.from_denom;
    let balance = deps
        .querier
        .query_balance(env.contract.address, from_denom.clone())?;

    if balance.amount < config.min_rewards {
        return Err(ContractError::LowBalance {
            min_amount: config.min_rewards,
            amount: balance.amount,
            denom: from_denom,
        });
    }

    let mut res = response(
        "exchange",
        CONTRACT_NAME,
        [attr_coin(
            "swap_amount",
            balance.amount,
            balance.denom.clone(),
        )],
    );

    if let Some(swap_operations) = swap_operations {
        let router_contract_address = config.router_contract;

        let exchange_rewards_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: router_contract_address.clone(),
            msg: to_json_binary(&RouterExecuteMsg::ExecuteSwapOperations {
                operations: swap_operations,
                minimum_receive: None,
                to: Some(config.core_contract),
                max_spread: None,
            })?,
            funds: vec![balance.clone()],
        });

        res = res
            .add_message(exchange_rewards_msg)
            .add_attribute("router_contract", router_contract_address);
    } else {
        let pair_contract = config.pair_contract;

        let exchange_rewards_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_contract.to_string(),
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
                to: Some(config.core_contract),
            })?,
            funds: vec![balance.clone()],
        });

        res = res
            .add_message(exchange_rewards_msg)
            .add_attribute("pair_contract", pair_contract);
    }

    Ok(res)
}

fn exec_update_swap_operations(
    deps: DepsMut,
    info: MessageInfo,
    operations: Option<Vec<SwapOperation>>,
) -> ContractResult<Response> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut attrs = vec![];
    if let Some(operations) = operations {
        SWAP_OPERATIONS.save(deps.storage, &operations)?;
        attrs.push(attr("new_swap_operations", operations.len().to_string()));
    } else {
        SWAP_OPERATIONS.remove(deps.storage);
        attrs.push(attr("clear_operations", "1".to_string()));
    }
    Ok(response(
        "update_swap_operations",
        CONTRACT_NAME,
        attrs,
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
