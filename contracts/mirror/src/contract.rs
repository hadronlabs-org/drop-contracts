use cosmwasm_std::{
    attr, ensure, from_json, to_json_binary, Attribute, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, IbcQuery, MessageInfo, Reply, Response, SubMsg, Uint128, WasmMsg,
};
use cw_ownable::update_ownership;
use drop_helpers::answer::response;
use drop_helpers::ibc_fee::query_ibc_fee;
use drop_staking_base::msg::mirror::{ExecuteMsg, FailedReceiverResponse, FungibleTokenPacketData};
use drop_staking_base::state::mirror::{
    Config, ConfigOptional, CONFIG, FAILED_TRANSFERS, FAILED_TRANSFER_REPLY_ID, REPLY_RECEIVER,
};
use drop_staking_base::{
    error::mirror::{ContractError, ContractResult},
    msg::mirror::{InstantiateMsg, MigrateMsg, QueryMsg},
    state::mirror::TIMEOUT_RANGE,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, TransferSudoMsg};

use std::str::FromStr;
use std::{env, vec};

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
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
    REPLY_RECEIVER.save(deps.storage, &"".to_string())?;
    let attrs = vec![attr("action", "instantiate"), attr("owner", owner)];
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => Ok(to_json_binary(&CONFIG.load(deps.storage)?)?),
        QueryMsg::FailedReceiver { receiver } => query_failed_receiver(deps, receiver),
        QueryMsg::AllFailed {} => query_all_failed(deps),
    }
}

fn query_all_failed(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let failed_transfers: Vec<(String, Vec<Coin>)> = FAILED_TRANSFERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|pair| {
            let (receiver, funds) = pair.unwrap();
            (receiver, funds)
        })
        .collect();
    Ok(to_json_binary(&failed_transfers)?)
}

fn query_failed_receiver(deps: Deps<NeutronQuery>, receiver: String) -> ContractResult<Binary> {
    let failed_transfers = FAILED_TRANSFERS.may_load(deps.storage, receiver.clone())?;
    if let Some(failed_transfers) = failed_transfers {
        return Ok(to_json_binary(&FailedReceiverResponse {
            receiver,
            amount: failed_transfers,
        })?);
    }
    Ok(to_json_binary::<Option<FailedReceiverResponse>>(&None)?)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond { receiver, r#ref } => execute_bond(deps, env, info, receiver, r#ref),
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::Retry { receiver } => execute_retry(deps, env, receiver),
    }
}

fn execute_retry(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    let failed_transfers = FAILED_TRANSFERS.may_load(deps.storage, receiver.clone())?;
    let Config {
        source_port,
        source_channel,
        ibc_timeout,
        ..
    } = CONFIG.load(deps.storage)?;
    let mut ibc_transfer_msgs: Vec<CosmosMsg<NeutronMsg>> = vec![];
    let mut attrs: Vec<Attribute> = vec![];
    if let Some(failed_transfers) = failed_transfers {
        for coin in failed_transfers.iter() {
            ibc_transfer_msgs.push(CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                source_port: source_port.clone(),
                source_channel: source_channel.clone(),
                token: coin.clone(),
                sender: env.contract.address.to_string(),
                receiver: receiver.clone(),
                timeout_height: RequestPacketTimeoutHeight {
                    revision_number: None,
                    revision_height: None,
                },
                timeout_timestamp: env.block.time.plus_seconds(ibc_timeout).nanos(),
                memo: "".to_string(),
                fee: query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?,
            }));
            attrs.push(attr("receiver", receiver.clone()));
            attrs.push(attr("amount", coin.to_string()));
        }
    }
    // During the IBC transfers we need to remove these funds from state so we can't call retry again
    // If any IBC transaction fails then we restore failed transfers in sudo
    FAILED_TRANSFERS.save(deps.storage, receiver, &vec![])?;
    Ok(response("retry", CONTRACT_NAME, attrs).add_messages(ibc_transfer_msgs))
}

fn execute_update_config(
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
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

pub fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    receiver: String,
    r#ref: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let Config {
        core_contract,
        prefix,
        ..
    } = CONFIG.load(deps.storage)?;
    ensure!(receiver.starts_with(&prefix), ContractError::InvalidPrefix);
    bech32::decode(&receiver).map_err(|_| ContractError::WrongReceiverAddress)?;
    let coin = cw_utils::one_coin(&info)?;
    let attrs = vec![
        attr("action", "bond"),
        attr("receiver", receiver.to_string()),
        attr("ref", r#ref.clone().unwrap_or_default()),
    ];
    REPLY_RECEIVER.save(deps.storage, &"".to_string())?;
    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: core_contract,
            msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Bond {
                receiver: None,
                r#ref,
            })?,
            funds: vec![coin],
        },
        FAILED_TRANSFER_REPLY_ID,
    );
    Ok(response("bond", CONTRACT_NAME, attrs).add_submessage(msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    finalize_bond(deps, env, msg)
}

pub fn finalize_bond(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(res) => {
            let Config {
                source_port,
                source_channel,
                ibc_timeout,
                ..
            } = CONFIG.load(deps.storage)?;
            let receiver = REPLY_RECEIVER.load(deps.storage)?;
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
            let attrs = vec![
                attr("action", "finalize_bond"),
                attr("id", msg.id.to_string()),
                attr("amount", coin.to_string()),
                attr("to_address", receiver.clone()),
            ];
            let ibc_transfer_msg: CosmosMsg<NeutronMsg> =
                CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                    source_port: source_port.clone(),
                    source_channel: source_channel.clone(),
                    token: coin, // at this point unwrap is safe as bond is finalized already
                    sender: env.contract.address.to_string(),
                    receiver: receiver.clone(),
                    timeout_height: RequestPacketTimeoutHeight {
                        revision_number: None,
                        revision_height: None,
                    },
                    timeout_timestamp: env.block.time.plus_seconds(ibc_timeout).nanos(),
                    memo: "".to_string(),
                    fee: query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?,
                });
            Ok(response("finalize_bond", CONTRACT_NAME, attrs).add_message(ibc_transfer_msg))
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
        TransferSudoMsg::Response { .. } => sudo_response(),
        TransferSudoMsg::Error { request, .. } => sudo_error(deps, request, "sudo-error"),
        TransferSudoMsg::Timeout { request } => sudo_error(deps, request, "sudo-timeout"),
    }
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    req: RequestPacket,
    ty: &str,
) -> ContractResult<Response<NeutronMsg>> {
    let packet: FungibleTokenPacketData = from_json(req.data.unwrap())?;
    let packet_amount = Uint128::from_str(packet.amount.as_str()).unwrap();

    FAILED_TRANSFERS.update(
        deps.storage,
        packet.receiver,
        |current_value| match current_value {
            Some(funds) => Ok::<Vec<Coin>, ContractError>(
                funds
                    .iter()
                    .map(|coin| {
                        if coin.denom == packet.denom {
                            // if sender already have pending failed transfer in this denom then just update the value
                            return Coin {
                                denom: coin.denom.clone(),
                                amount: coin.amount + packet_amount,
                            };
                        }
                        coin.clone()
                    })
                    .collect::<Vec<Coin>>(),
            ),
            None => Ok(vec![Coin {
                denom: packet.denom,
                amount: packet_amount,
            }]), // if to_address don't have any pending coins yet
        },
    )?;

    Ok(response(ty, CONTRACT_NAME, Vec::<Attribute>::new()))
}

fn sudo_response() -> ContractResult<Response<NeutronMsg>> {
    Ok(response(
        "sudo_response",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let contract_version_metadata = cw2::get_contract_version(deps.storage)?;
    let storage_contract_name = contract_version_metadata.contract.as_str();
    if storage_contract_name != CONTRACT_NAME {
        return Err(ContractError::MigrationError {
            storage_contract_name: storage_contract_name.to_string(),
            contract_name: CONTRACT_NAME.to_string(),
        });
    }

    let storage_version: semver::Version = contract_version_metadata.version.parse()?;
    let version: semver::Version = CONTRACT_VERSION.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }
    Ok(Response::new())
}
