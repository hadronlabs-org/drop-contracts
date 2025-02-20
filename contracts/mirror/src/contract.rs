use crate::error::{ContractError, ContractResult};
use crate::msg::{
    ExecuteMsg, FailedReceiverResponse, FungibleTokenPacketData, InstantiateMsg, MigrateMsg,
    QueryMsg,
};
use crate::state::{
    Config, ConfigOptional, BOND_REPLY_ID, BOND_REPLY_RECEIVER, CONFIG, FAILED_TRANSFERS,
    IBC_TRANSFER_SUDO_REPLY_ID, REPLY_TRANSFER_COINS, SUDO_SEQ_ID_TO_COIN, TIMEOUT_RANGE,
};
use cosmwasm_std::{
    attr, ensure, from_json, to_json_binary, Attribute, Binary, Coin, Deps, DepsMut, Env, IbcQuery,
    MessageInfo, Reply, Response, StdError, SubMsg, Uint128, WasmMsg,
};
use cw_ownable::update_ownership;
use drop_helpers::answer::response;
use drop_helpers::ibc_fee::query_ibc_fee;
use neutron_sdk::bindings::{
    msg::{MsgIbcTransferResponse, NeutronMsg},
    query::NeutronQuery,
};
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, TransferSudoMsg};

use std::collections::VecDeque;
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
            core_contract: msg.core_contract.clone(),
            source_port: msg.source_port.clone(),
            source_channel: msg.source_channel.clone(),
            ibc_timeout: msg.ibc_timeout,
            prefix: msg.prefix.clone(),
            retry_limit: msg.retry_limit,
        },
    )?;
    REPLY_TRANSFER_COINS.save(deps.storage, &VecDeque::new())?;
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
        QueryMsg::FailedReceiver { receiver } => query_failed_receiver(deps, receiver),
        QueryMsg::AllFailed {} => query_all_failed(deps),
    }
}

fn query_all_failed(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let failed_transfers: Vec<FailedReceiverResponse> = FAILED_TRANSFERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|pair| {
            let (r, d) = pair.unwrap(); // safe because it's a range map
            FailedReceiverResponse {
                receiver: r,
                debt: d,
            }
        })
        .collect();
    Ok(to_json_binary(&failed_transfers)?)
}

fn query_failed_receiver(deps: Deps<NeutronQuery>, receiver: String) -> ContractResult<Binary> {
    let failed_transfers = FAILED_TRANSFERS.may_load(deps.storage, receiver.clone())?;
    if let Some(failed_transfers) = failed_transfers {
        return Ok(to_json_binary(&Some(FailedReceiverResponse {
            receiver,
            debt: failed_transfers,
        }))?);
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
        retry_limit,
        ..
    } = CONFIG.load(deps.storage)?;
    let mut ibc_transfer_submsgs: Vec<SubMsg<NeutronMsg>> = vec![];
    let mut attrs: Vec<Attribute> = vec![attr("action", "execute_retry")];
    if let Some(failed_transfers) = failed_transfers {
        let mut receiver_new_coins: Vec<Coin> = failed_transfers.clone();
        for coin in failed_transfers
            .iter()
            .take(retry_limit.try_into().unwrap())
        {
            ibc_transfer_submsgs.push(SubMsg::reply_on_success(
                NeutronMsg::IbcTransfer {
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
                },
                IBC_TRANSFER_SUDO_REPLY_ID,
            ));
            receiver_new_coins = receiver_new_coins
                .iter()
                .filter(|receiver_new_coin| receiver_new_coin.denom != coin.denom)
                .cloned()
                .collect::<Vec<Coin>>();
            REPLY_TRANSFER_COINS.update(deps.storage, |mut reply_transfer_coins| {
                reply_transfer_coins.push_back(coin.clone());
                Ok::<VecDeque<Coin>, ContractError>(reply_transfer_coins)
            })?;
            attrs.push(attr("receiver", receiver.clone()));
            attrs.push(attr("amount", coin.to_string()));
        }
        // During the IBC transfers we need to remove these funds from state so we can't call retry again for the same user
        // If any IBC transaction fails then we restore failed transfers for given user in sudo-error
        // It doesn't throw any exception if given key doesn't exist
        if receiver_new_coins.is_empty() {
            FAILED_TRANSFERS.remove(deps.storage, receiver);
        } else {
            FAILED_TRANSFERS.save(deps.storage, receiver, &receiver_new_coins)?;
        }
    }
    Ok(response("execute_retry", CONTRACT_NAME, attrs).add_submessages(ibc_transfer_submsgs))
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
        attrs.push(attr("retry_limit", retry_limit.to_string()));
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
    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: core_contract,
            msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Bond {
                receiver: None,
                r#ref: r#ref.clone(),
            })?,
            funds: vec![coin.clone()],
        },
        BOND_REPLY_ID,
    );
    let attrs = vec![
        attr("action", "execute_bond"),
        attr("receiver", receiver.to_string()),
        attr("ref", r#ref.clone().unwrap_or_default()),
        attr("coin", coin.to_string()),
    ];
    // We can't pass receiver directly to reply from bond execution
    // The only way to pass it is to overwrite receiver here and then read in reply
    BOND_REPLY_RECEIVER.save(deps.storage, &receiver)?;
    Ok(response("execute_bond", CONTRACT_NAME, attrs).add_submessage(msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.id {
        BOND_REPLY_ID => finalize_bond(deps, env, msg),
        IBC_TRANSFER_SUDO_REPLY_ID => store_seq_id(deps, msg),
        _ => unimplemented!(),
    }
}

pub fn store_seq_id(
    deps: DepsMut<NeutronQuery>,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    let msg_ibc_transfer_response: MsgIbcTransferResponse = serde_json_wasm::from_slice(
        msg.result
            .into_result()
            .map_err(StdError::generic_err)?
            .data
            .ok_or_else(|| StdError::generic_err("no result"))?
            .as_slice(),
    )
    .map_err(|e| StdError::generic_err(format!("failed to parse response: {e:?}")))?;
    let seq_id = msg_ibc_transfer_response.sequence_id;
    let mut coins = REPLY_TRANSFER_COINS.load(deps.storage)?;
    let coin = coins.pop_front().unwrap(); // safe because it always has something inside
    REPLY_TRANSFER_COINS.save(deps.storage, &coins)?;
    SUDO_SEQ_ID_TO_COIN.save(deps.storage, seq_id, &coin)?;
    let attrs = vec![
        attr("action", "store_seq_id"),
        attr("popped", coin.to_string()),
    ];
    Ok(response("reply_store_seq_id", CONTRACT_NAME, attrs))
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
            let receiver = BOND_REPLY_RECEIVER.load(deps.storage)?;
            let tf_mint_event = res
                .events
                .iter()
                .find(|x| x.ty == "tf_mint")
                .ok_or(ContractError::NoTokensMinted)?;
            let coin = Coin::from_str(
                &tf_mint_event
                    .attributes
                    .iter()
                    .find(|x| x.key == "amount")
                    .ok_or(ContractError::NoTokensMintedAmountFound)?
                    .value,
            )?;
            let attrs = vec![
                attr("action", "reply_finalize_bond"),
                attr("amount", coin.to_string()),
                attr("to_address", receiver.clone()),
                attr("source_port", source_port.to_string()),
                attr("source_channel", source_channel.clone()),
                attr("ibc-timeout", ibc_timeout.to_string()),
            ];
            // Send all tokens that we got from the bond action back to the remote chain
            let ibc_transfer_submsg: SubMsg<NeutronMsg> = SubMsg::reply_on_success(
                NeutronMsg::IbcTransfer {
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
                },
                IBC_TRANSFER_SUDO_REPLY_ID,
            );
            REPLY_TRANSFER_COINS.update(deps.storage, |mut reply_transfer_coins| {
                reply_transfer_coins.push_back(coin);
                Ok::<VecDeque<Coin>, ContractError>(reply_transfer_coins)
            })?;
            Ok(response("reply_finalize_bond", CONTRACT_NAME, attrs)
                .add_submessage(ibc_transfer_submsg))
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
        TransferSudoMsg::Response { request, .. } => sudo_response(deps, request),
        TransferSudoMsg::Error { request, .. } => sudo_error(deps, request, "sudo_error"),
        TransferSudoMsg::Timeout { request } => sudo_error(deps, request, "sudo_timeout"),
    }
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    req: RequestPacket,
    ty: &str,
) -> ContractResult<Response<NeutronMsg>> {
    // https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md#data-structures
    // Unfortunately, ics-20 says that if you transfer IBC denom, then in sudo handler you will get back a trace
    // instead of the original denomination. 123ibc/... -> 123transfer/channel-0/denom. In order to handle this
    // we're using a SUDO_SEQ_ID_TO_COIN map, where we store an IBC transfer sequence id with a denomination.
    // We get sequence id from the reply handler, where IBC transfer message is constructed. And then unwrap it here
    let seq_id = req.sequence.unwrap();
    let actual_denom = SUDO_SEQ_ID_TO_COIN.load(deps.storage, seq_id)?.denom;
    let packet: FungibleTokenPacketData = from_json(req.data.unwrap())?;
    let packet_amount = Uint128::from_str(packet.amount.as_str())?;

    // If given ibc-transfer for given receiver on the remote chain fails then
    // current contract owns these tokens right now. Memorize in the map, that
    // for given user our contract obtains these failed-to-process tokens
    FAILED_TRANSFERS.update(deps.storage, packet.receiver, |current_debt| {
        let mut new_debt = current_debt.unwrap_or_default();
        new_debt.push(Coin {
            denom: actual_denom,
            amount: packet_amount,
        });
        Ok::<Vec<Coin>, ContractError>(new_debt)
    })?;
    SUDO_SEQ_ID_TO_COIN.remove(deps.storage, seq_id);
    Ok(response(ty, CONTRACT_NAME, Vec::<Attribute>::new()))
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    req: RequestPacket,
) -> ContractResult<Response<NeutronMsg>> {
    let seq_id = req.sequence.unwrap();
    SUDO_SEQ_ID_TO_COIN.remove(deps.storage, seq_id);
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
    Ok(Response::new())
}
