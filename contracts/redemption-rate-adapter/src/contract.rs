use cosmwasm_std::{attr, from_json, to_json_binary, Attribute, Decimal, Deps};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::response;
use drop_staking_base::error::redemption_rate_adapter::{ContractError, ContractResult};
use drop_staking_base::msg::redemption_rate_adapter::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, RedemptionRateResponse, UpdateConfig,
};

use drop_staking_base::state::redemtion_rate_adapter::{Config, CONFIG};
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

    let core_contract = deps.api.addr_validate(&msg.core_contract)?;
    let config = &Config {
        core_contract: core_contract.clone(),
        denom: msg.denom,
    };
    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("core_contract", core_contract)],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::RedemptionRate { denom, .. } => query_redemption_rate(deps, env, denom),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_redemption_rate(
    deps: Deps<NeutronQuery>,
    env: Env,
    denom: String,
) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    if denom != config.denom {
        return Err(ContractError::InvalidDenom {});
    }
    let exchange_rate: Decimal = deps.querier.query_wasm_smart(
        config.core_contract.clone(),
        &drop_staking_base::msg::core::QueryMsg::ExchangeRate {},
    )?;
    let core_state: drop_staking_base::state::core::ContractState = deps.querier.query_wasm_smart(
        config.core_contract.clone(),
        &drop_staking_base::msg::core::QueryMsg::ContractState {},
    )?;

    let update_time = match core_state {
        drop_staking_base::state::core::ContractState::Idle => env.block.time.seconds(),
        _ => {
            let last_idle_raw = deps
                .querier
                .query_wasm_raw(config.core_contract, b"last_tick")?
                .ok_or_else(|| {
                    ContractError::Std(cosmwasm_std::StdError::NotFound {
                        kind: "last_tick".to_string(),
                    })
                })?;
            from_json::<u64>(last_idle_raw)?
        }
    };

    Ok(to_json_binary(&RedemptionRateResponse {
        redemption_rate: exchange_rate,
        update_time,
    })?)
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
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: UpdateConfig,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut attrs = vec![];
    let new_core_contract = deps.api.addr_validate(&new_config.core_contract)?;

    attrs.push(attr("core_contract", new_config.core_contract.to_string()));
    attrs.push(attr("denom", new_config.denom.to_string()));

    CONFIG.save(
        deps.storage,
        &Config {
            core_contract: new_core_contract,
            denom: new_config.denom,
        },
    )?;

    Ok(response("update_config", CONTRACT_NAME, Vec::<Attribute>::new()).add_attributes(attrs))
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
