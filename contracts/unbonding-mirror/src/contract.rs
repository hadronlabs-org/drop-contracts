use crate::error::{ContractError, ContractResult};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, ConfigOptional, CONFIG, TIMEOUT_RANGE};
use cosmwasm_std::{
    attr, to_json_binary, Binary, Deps, DepsMut, Env, IbcQuery, MessageInfo, Response,
};
use drop_helpers::answer::response;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use std::env;

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const _LOCAL_DENOM: &str = "untrn";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;
    deps.api.addr_validate(&msg.core_contract)?;
    CONFIG.save(
        deps.storage,
        &Config {
            core_contract: msg.core_contract.clone(),
            source_port: msg.source_port.clone(),
            source_channel: msg.source_channel.clone(),
            ibc_timeout: msg.ibc_timeout.clone(),
            prefix: msg.prefix.clone(),
            retry_limit: msg.retry_limit.clone(),
        },
    )?;
    let attrs = vec![
        attr("action", "instantiate"),
        attr("owner", owner),
        attr("core_contract", msg.core_contract),
        attr("source_port", msg.source_port),
        attr("source_channel", msg.source_channel),
        attr("ibc_timeout", msg.ibc_timeout.to_string()),
        attr("prefix", msg.prefix),
        attr("retry_limit", msg.retry_limit.to_string()),
    ];
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => Ok(to_json_binary(&CONFIG.load(deps.storage)?)?),
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
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut attrs = vec![attr("action", "execute_update_config")];
    if let Some(retry_limit) = new_config.retry_limit {
        attrs.push(attr("retry_limit", &retry_limit.to_string()));
        config.retry_limit = retry_limit;
    }
    if let Some(core_contract) = new_config.core_contract {
        deps.api.addr_validate(&core_contract)?;
        attrs.push(attr("core_contract", &core_contract));
        config.core_contract = core_contract;
    }
    if let Some(ibc_timeout) = new_config.ibc_timeout {
        if !(TIMEOUT_RANGE.from..=TIMEOUT_RANGE.to).contains(&ibc_timeout) {
            return Err(ContractError::IbcTimeoutOutOfRange);
        }
        attrs.push(attr("ibc_timeout", ibc_timeout.to_string()));
        config.ibc_timeout = ibc_timeout;
    }
    if let Some(prefix) = new_config.prefix {
        attrs.push(attr("prefix", &prefix));
        config.prefix = prefix;
    }
    {
        if let Some(source_port) = new_config.source_port {
            attrs.push(attr("source_port", &source_port));
            config.source_port = source_port;
        }
        if let Some(source_channel) = new_config.source_channel {
            attrs.push(attr("source_channel", &source_channel));
            config.source_channel = source_channel;
        }
        let res: cosmwasm_std::ChannelResponse = deps
            .querier
            .query(&cosmwasm_std::QueryRequest::Ibc(IbcQuery::Channel {
                channel_id: config.source_channel.clone(),
                port_id: Some(config.source_port.clone()),
            }))
            .unwrap();
        if res.channel.is_none() {
            return Err(ContractError::SourceChannelNotFound);
        }
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("execute_update_config", CONTRACT_NAME, attrs))
}
