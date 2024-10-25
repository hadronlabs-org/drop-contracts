use cosmwasm_std::{
    attr, ensure, ensure_eq, from_json, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Order, Reply, Response, SubMsg, WasmMsg,
};
use cw_ownable::update_ownership;
use drop_helpers::answer::response;
use drop_helpers::ibc_fee::query_ibc_fee;
use drop_staking_base::msg::mirror::{ExecuteMsg, FungibleTokenPacketData};
use drop_staking_base::state::mirror::{
    BondItem, BondState, Config, ConfigOptional, ReturnType, BONDS, CONFIG, COUNTER,
};
use drop_staking_base::{
    error::mirror::{ContractError, ContractResult},
    msg::mirror::{InstantiateMsg, MigrateMsg, QueryMsg},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, TransferSudoMsg};

use std::marker::PhantomData;
use std::str::FromStr;
use std::{env, vec};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const LOCAL_DENOM: &str = "untrn";

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
            core_contract: msg.core_contract,
            source_port: msg.source_port,
            source_channel: msg.source_channel,
            ibc_timeout: msg.ibc_timeout,
            prefix: msg.prefix,
        },
    )?;
    COUNTER.save(deps.storage, &0)?;
    let attrs = vec![attr("action", "instantiate"), attr("owner", owner)];
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => Ok(to_json_binary(&CONFIG.load(deps.storage)?)?),
        QueryMsg::One { id } => query_one(deps, id),
        QueryMsg::All { start_after, limit } => query_all(deps, start_after, limit),
    }
}

pub fn query_one(deps: Deps<NeutronQuery>, id: u64) -> ContractResult<Binary> {
    let bond = BONDS.load(deps.storage, id)?;
    to_json_binary(&bond).map_err(ContractError::Std)
}

pub fn query_all(
    deps: Deps<NeutronQuery>,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> ContractResult<Binary> {
    let limit = limit.map(|x| x as usize).unwrap_or(usize::MAX);
    let bonds = BONDS
        .range(
            deps.storage,
            start_after.map(|x| cw_storage_plus::Bound::Inclusive((x, PhantomData))),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|item| item.map_err(ContractError::Std))
        .collect::<ContractResult<Vec<_>>>()?;
    to_json_binary(&bonds).map_err(ContractError::Std)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond {
            receiver,
            backup,
            r#ref,
        } => execute_bond(deps, env, info, receiver, r#ref, backup),
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::Complete { items } => execute_complete(deps, env, info, items),
        ExecuteMsg::ChangeReturnType { id, return_type } => {
            execute_change_return_type(deps, env, info, id, return_type)
        }
        ExecuteMsg::UpdateBond {
            id,
            receiver,
            backup,
            return_type,
        } => execute_update_bond(deps, env, info, id, receiver, backup, return_type),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
    }
}

pub fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut config = CONFIG.load(deps.storage)?;
    let mut attrs = vec![attr("action", "update_config")];
    if let Some(core_contract) = new_config.core_contract {
        deps.api.addr_validate(&core_contract)?;
        attrs.push(attr("core_contract", &core_contract));
        config.core_contract = core_contract;
    }
    if let Some(source_port) = new_config.source_port {
        attrs.push(attr("source_port", &source_port));
        config.source_port = source_port;
    }
    if let Some(source_channel) = new_config.source_channel {
        attrs.push(attr("source_channel", &source_channel));
        config.source_channel = source_channel;
    }
    if let Some(ibc_timeout) = new_config.ibc_timeout {
        attrs.push(attr("ibc_timeout", ibc_timeout.to_string()));
        config.ibc_timeout = ibc_timeout;
    }
    if let Some(prefix) = new_config.prefix {
        attrs.push(attr("prefix", &prefix));
        config.prefix = prefix;
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

pub fn execute_change_return_type(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    id: u64,
    return_type: ReturnType,
) -> ContractResult<Response<NeutronMsg>> {
    let mut bond = BONDS.load(deps.storage, id)?;
    ensure!(
        bond.state == BondState::Bonded,
        ContractError::WrongBondState {
            expected: BondState::Bonded.to_string(),
            got: bond.state.to_string(),
        }
    );
    let backup = bond.backup.clone().ok_or(ContractError::BackupIsNotSet)?;

    ensure_eq!(info.sender, backup, ContractError::Unauthorized);
    bond.return_type = return_type.clone();
    BONDS.save(deps.storage, id, &bond)?;
    let attrs = vec![
        attr("action", "change_return_type"),
        attr("id", id.to_string()),
        attr("return_type", return_type.to_string()),
    ];
    Ok(response("change_return_type", CONTRACT_NAME, attrs))
}

pub fn execute_update_bond(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    id: u64,
    receiver: String,
    backup: Option<String>,
    return_type: ReturnType,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut bond = BONDS.load(deps.storage, id)?;
    bond.receiver = receiver.clone();
    bond.backup = backup
        .clone()
        .map(|a| deps.api.addr_validate(&a))
        .transpose()?;
    bond.return_type = return_type.clone();
    BONDS.save(deps.storage, id, &bond)?;
    let attrs = vec![
        attr("action", "update_bond"),
        attr("id", id.to_string()),
        attr("receiver", receiver),
        attr("backup", backup.unwrap_or_default()),
        attr("return_type", return_type.to_string()),
    ];
    Ok(response("update_bond_state", CONTRACT_NAME, attrs))
}

pub fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    receiver: String,
    r#ref: Option<String>,
    backup: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let Config {
        core_contract,
        prefix,
        ..
    } = CONFIG.load(deps.storage)?;
    let backup = backup
        .as_ref()
        .map(|addr| deps.api.addr_validate(addr))
        .transpose()?;
    bech32::decode(&receiver).map_err(|_| ContractError::WrongReceiverAddress)?;
    ensure!(receiver.starts_with(&prefix), ContractError::InvalidPrefix);
    let coin = cw_utils::one_coin(&info)?;
    let counter = COUNTER.load(deps.storage)?;
    let id = counter + 1;
    COUNTER.save(deps.storage, &id)?;
    let attrs = vec![
        attr("action", "bond"),
        attr("id", id.to_string()),
        attr("receiver", receiver.to_string()),
        attr("ref", r#ref.clone().unwrap_or_default()),
        attr(
            "backup",
            backup.as_ref().map(|x| x.to_string()).unwrap_or_default(),
        ),
    ];
    BONDS.save(
        deps.storage,
        id,
        &BondItem {
            receiver,
            backup,
            received: None,
            amount: coin.amount,
            return_type: drop_staking_base::state::mirror::ReturnType::default(),
            state: drop_staking_base::state::mirror::BondState::default(),
        },
    )?;
    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: core_contract,
            msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Bond {
                receiver: None,
                r#ref,
            })?,
            funds: vec![coin],
        },
        id,
    );
    Ok(response("bond", CONTRACT_NAME, attrs).add_submessage(msg))
}

pub fn execute_complete(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    items: Vec<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let mut msgs = vec![];
    let mut attrs = vec![attr("action", "complete")];
    let Config {
        source_port,
        source_channel,
        ibc_timeout,
        ..
    } = CONFIG.load(deps.storage)?;
    for id in items {
        let mut bond = BONDS.load(deps.storage, id)?;
        attrs.push(attr("id", id.to_string()));
        attrs.push(attr("return_type", bond.state.to_string()));
        attrs.push(attr("coin", bond.received.clone().unwrap().to_string())); // at this point unwrap is safe as bond is finalized already
        ensure_eq!(
            bond.state,
            BondState::Bonded,
            ContractError::WrongBondState {
                expected: BondState::Bonded.to_string(),
                got: bond.state.to_string(),
            }
        );
        match bond.return_type {
            ReturnType::Remote => {
                bond.state = BondState::Sent;
                BONDS.save(deps.storage, id, &bond)?;
                msgs.push(CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: source_port.clone(),
                    source_channel: source_channel.clone(),
                    token: bond.received.unwrap(), // at this point unwrap is safe as bond is finalized already
                    sender: env.contract.address.to_string(),
                    receiver: bond.receiver,
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None,
                    },
                    timeout_timestamp: env.block.time.plus_seconds(ibc_timeout).nanos(),
                    memo: id.to_string(),
                    fee: query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?,
                }));
            }
            ReturnType::Local => {
                if let Some(backup) = bond.backup {
                    BONDS.remove(deps.storage, id);
                    msgs.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: backup.to_string(),
                        amount: vec![bond.received.unwrap()],
                    }));
                }
            }
        }
    }
    Ok(response("complete", CONTRACT_NAME, attrs).add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    finalize_bond(deps, msg)
}

pub fn finalize_bond(
    deps: DepsMut<NeutronQuery>,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(res) => {
            let mut bond = BONDS.load(deps.storage, msg.id)?;
            bond.state = BondState::Bonded;
            // get token factory mint event
            let tf_mint_event = res
                .events
                .iter()
                .find(|x| x.ty == "tf_mint")
                .ok_or(ContractError::NoTokensMinted)?;
            // get amount from mint event
            let coin = Coin::from_str(
                &tf_mint_event
                    .attributes
                    .iter()
                    .find(|x| x.key == "amount")
                    .ok_or(ContractError::NoTokensMintedAmountFound)?
                    .value,
            )?;
            bond.received = Some(coin);
            BONDS.save(deps.storage, msg.id, &bond)?;
            let attrs = vec![
                attr("action", "finalize_bond"),
                attr("id", msg.id.to_string()),
                attr("state", bond.state.to_string()),
            ];
            Ok(response("finalize_bond", CONTRACT_NAME, attrs))
        }
        cosmwasm_std::SubMsgResult::Err(_) => unreachable!(), // as there is only SubMsg::reply_on_success()
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    msg: TransferSudoMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        TransferSudoMsg::Response { request, data } => sudo_response(deps, request, data),
        TransferSudoMsg::Error { request, details } => sudo_error(deps, request, details),
        TransferSudoMsg::Timeout { request } => sudo_timeout(deps, request),
    }
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    req: RequestPacket,
    _data: String,
) -> ContractResult<Response<NeutronMsg>> {
    let id = get_id_from_request_memo(&req)?;
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_error: ack received: {:?} id: {:?}",
            req, id
        )
        .as_str(),
    );
    let mut bond = BONDS.load(deps.storage, id)?;
    bond.state = BondState::Bonded;
    BONDS.save(deps.storage, id, &bond)?;
    let attrs = vec![
        attr("action", "sudo_error"),
        attr("id", id.to_string()),
        attr("state", bond.state.to_string()),
    ];
    Ok(response("sudo_error", CONTRACT_NAME, attrs))
}

fn sudo_timeout(
    deps: DepsMut<NeutronQuery>,
    req: RequestPacket,
) -> ContractResult<Response<NeutronMsg>> {
    let id = get_id_from_request_memo(&req)?;
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_timeout: ack received: {:?} id: {:?}",
            req, id
        )
        .as_str(),
    );
    let mut bond = BONDS.load(deps.storage, id)?;
    bond.state = BondState::Bonded;
    BONDS.save(deps.storage, id, &bond)?;
    let attrs = vec![
        attr("action", "sudo_timeout"),
        attr("id", id.to_string()),
        attr("state", bond.state.to_string()),
    ];
    Ok(response("sudo_timeout", CONTRACT_NAME, attrs))
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    req: RequestPacket,
    data: Binary,
) -> ContractResult<Response<NeutronMsg>> {
    let request_data: FungibleTokenPacketData = from_json(req.data.clone().unwrap())?; // must present as there is only IBC transfer for this contract
    let id: u64 = get_id_from_request_memo(&req)?;
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_response: sudo received: {:?} {:?} {:?} id: {:?}",
            req, data, request_data, id
        )
        .as_str(),
    );
    BONDS.remove(deps.storage, id);
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request", format!("{:?}", req)),
        attr("id", id.to_string()),
    ];
    Ok(response("sudo_response", CONTRACT_NAME, attrs))
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

fn get_id_from_request_memo(req: &RequestPacket) -> ContractResult<u64> {
    let request_data: FungibleTokenPacketData = from_json(req.data.clone().unwrap())?; // must present as there is only IBC transfer for this contract
    let id: u64 = request_data
        .memo
        .parse()
        .map_err(|_| ContractError::InvalidMemo)?;
    Ok(id)
}
