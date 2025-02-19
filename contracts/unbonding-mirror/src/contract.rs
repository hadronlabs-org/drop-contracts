use crate::error::{ContractError, ContractResult};
use crate::msg::{
    ExecuteMsg, FailedReceiverResponse, FungibleTokenPacketData, InstantiateMsg, QueryMsg,
};
use crate::state::{
    Config, ConfigOptional, CONFIG, FAILED_TRANSFERS, IBC_TRANSFER_SUDO_REPLY_ID, REPLY_RECEIVERS,
    REPLY_TRANSFER_COIN, SUDO_SEQ_ID_TO_COIN, TF_DENOM_TO_NFT_ID, TIMEOUT_RANGE, UNBOND_REPLY_ID,
    WITHDRAW_REPLY_ID,
};
use cosmwasm_std::{
    attr, ensure, from_json, to_json_binary, Attribute, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, IbcQuery, MessageInfo, Reply, Response, StdError, SubMsg, Uint128, WasmMsg,
};
use drop_helpers::answer::response;
use drop_helpers::ibc_fee::query_ibc_fee;
use neutron_sdk::bindings::{
    msg::{MsgIbcTransferResponse, NeutronMsg},
    query::NeutronQuery,
};
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, TransferSudoMsg};
use std::env;
use std::str::FromStr;

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
    deps.api.addr_validate(&msg.withdrawal_manager)?;
    deps.api.addr_validate(&msg.withdrawal_voucher)?;
    let channel_response: cosmwasm_std::ChannelResponse = deps
        .querier
        .query(&cosmwasm_std::QueryRequest::Ibc(IbcQuery::Channel {
            channel_id: msg.source_channel.clone(),
            port_id: Some(msg.source_port.clone()),
        }))
        .unwrap();
    if channel_response.channel.is_none() {
        return Err(ContractError::SourceChannelNotFound);
    }
    CONFIG.save(
        deps.storage,
        &Config {
            core_contract: msg.core_contract.clone(),
            withdrawal_manager: msg.withdrawal_manager.clone(),
            withdrawal_voucher: msg.withdrawal_voucher.clone(),
            source_port: msg.source_port.clone(),
            source_channel: msg.source_channel.clone(),
            ibc_timeout: msg.ibc_timeout,
            prefix: msg.prefix.clone(),
            retry_limit: msg.retry_limit,
        },
    )?;
    UNBOND_REPLY_ID.save(deps.storage, &0u64)?;
    WITHDRAW_REPLY_ID.save(deps.storage, &(u32::MAX as u64))?;
    let attrs = vec![
        attr("action", "instantiate"),
        attr("owner", owner),
        attr("core_contract", msg.core_contract),
        attr("withdrawal_manager", msg.withdrawal_manager),
        attr("withdrawal_voucher", msg.withdrawal_voucher),
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
        QueryMsg::UnbondReady { id } => query_unbond_ready(deps, id),
    }
}

fn query_failed_receiver(deps: Deps<NeutronQuery>, receiver: String) -> ContractResult<Binary> {
    let failed_transfers = FAILED_TRANSFERS.may_load(deps.storage, receiver.clone())?;
    if let Some(failed_transfers) = failed_transfers {
        return Ok(to_json_binary(&Some(FailedReceiverResponse {
            receiver,
            amount: failed_transfers,
        }))?);
    }
    Ok(to_json_binary::<Option<FailedReceiverResponse>>(&None)?)
}

fn query_all_failed(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let failed_transfers: Vec<(String, Vec<Coin>)> = FAILED_TRANSFERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|pair| pair.unwrap())
        .collect();
    Ok(to_json_binary(&failed_transfers)?)
}

fn query_unbond_ready(deps: Deps<NeutronQuery>, id: String) -> ContractResult<Binary> {
    let Config {
        withdrawal_voucher,
        core_contract,
        ..
    } = CONFIG.load(deps.storage)?;
    let nft_id = TF_DENOM_TO_NFT_ID.load(deps.storage, id)?;
    let nft_response: cw721::AllNftInfoResponse<
        drop_staking_base::msg::withdrawal_voucher::Extension,
    > = deps.querier.query_wasm_smart(
        withdrawal_voucher.clone(),
        &drop_staking_base::msg::withdrawal_voucher::QueryMsg::AllNftInfo {
            token_id: nft_id.clone(),
            include_expired: None,
        },
    )?;
    let batch_id = nft_response.info.extension.unwrap().batch_id;
    let batch_info: drop_staking_base::state::core::UnbondBatch = deps
        .querier
        .query_wasm_smart(
            core_contract.clone(),
            &drop_staking_base::msg::core::QueryMsg::UnbondBatch {
                batch_id: Uint128::from_str(batch_id.as_str())?,
            },
        )
        .unwrap();
    let batch_status = batch_info.status;
    Ok(to_json_binary(
        &(batch_status == drop_staking_base::state::core::UnbondBatchStatus::Withdrawn),
    )?)
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
        ExecuteMsg::Unbond { receiver } => execute_unbond(deps, info, receiver),
        ExecuteMsg::Retry { receiver } => execute_retry(deps, env, receiver),
        ExecuteMsg::Withdraw { receiver } => execute_withdraw(deps, info, receiver),
    }
}

fn execute_withdraw(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    let coin = cw_utils::one_coin(&info)?;
    let Config {
        withdrawal_manager,
        withdrawal_voucher,
        prefix,
        ..
    } = CONFIG.load(deps.storage)?;

    ensure!(receiver.starts_with(&prefix), ContractError::InvalidPrefix);
    bech32::decode(&receiver).map_err(|_| ContractError::WrongReceiverAddress)?;

    let nft_id = TF_DENOM_TO_NFT_ID.load(deps.storage, coin.denom.clone())?;
    let withdraw_reply_id: u64 = WITHDRAW_REPLY_ID.load(deps.storage)? + 1;
    let withdraw_submsg: SubMsg<NeutronMsg> = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: withdrawal_voucher.clone(),
            msg: to_json_binary(
                &drop_staking_base::msg::withdrawal_voucher::ExecuteMsg::SendNft {
                    contract: withdrawal_manager.clone(),
                    token_id: nft_id,
                    msg: to_json_binary(
                        &drop_staking_base::msg::withdrawal_manager::ReceiveNftMsg::Withdraw {
                            receiver: None,
                        },
                    )?,
                },
            )?,
            funds: vec![],
        }),
        withdraw_reply_id,
    );
    let tf_burn_msg = CosmosMsg::Custom(NeutronMsg::submit_burn_tokens(
        coin.denom.clone(),
        coin.amount,
    ));
    WITHDRAW_REPLY_ID.save(deps.storage, &withdraw_reply_id)?;
    REPLY_RECEIVERS.save(deps.storage, withdraw_reply_id, &receiver)?;
    // we successfully exchange tf denom back to NFT and NFT to native assets.
    // No need to keep this relation anymore
    TF_DENOM_TO_NFT_ID.remove(deps.storage, coin.denom.clone());
    let attrs: Vec<Attribute> = vec![
        attr("action", "execute_withdraw"),
        attr("voucher_amount", coin.to_string()),
        attr("withdrawal_manager", withdrawal_manager),
        attr("withdrawal_voucher", withdrawal_voucher),
        attr("withdraw_reply_id", withdraw_reply_id.to_string()),
        attr("burn", coin.to_string()),
    ];
    Ok(response("execute_withdraw", CONTRACT_NAME, attrs)
        .add_message(tf_burn_msg)
        .add_submessage(withdraw_submsg))
}

fn execute_retry(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    let Config {
        source_port,
        source_channel,
        ibc_timeout,
        retry_limit,
        prefix,
        ..
    } = CONFIG.load(deps.storage)?;

    ensure!(receiver.starts_with(&prefix), ContractError::InvalidPrefix);
    bech32::decode(&receiver).map_err(|_| ContractError::WrongReceiverAddress)?;

    let failed_transfers = FAILED_TRANSFERS.may_load(deps.storage, receiver.clone())?;
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
            attrs.push(attr("receiver", receiver.clone()));
            attrs.push(attr("amount", coin.to_string()));
        }
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
    if let Some(withdrawal_manager) = new_config.withdrawal_manager {
        deps.api.addr_validate(&withdrawal_manager)?;
        attrs.push(attr("withdrawal_manager", &withdrawal_manager));
        config.withdrawal_manager = withdrawal_manager;
    }
    if let Some(withdrawal_voucher) = new_config.withdrawal_voucher {
        deps.api.addr_validate(&withdrawal_voucher)?;
        attrs.push(attr("withdrawal_voucher", &withdrawal_voucher));
        config.withdrawal_voucher = withdrawal_voucher;
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

fn execute_unbond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    let coin = cw_utils::one_coin(&info)?;
    let config = CONFIG.load(deps.storage)?;

    ensure!(
        receiver.starts_with(&config.prefix),
        ContractError::InvalidPrefix
    );
    bech32::decode(&receiver).map_err(|_| ContractError::WrongReceiverAddress)?;

    // We can't pass receiver directly to reply from unbond execution
    // The only way to pass it is to overwrite receiver here and then read in reply
    let unbond_reply_id: u64 = UNBOND_REPLY_ID.load(deps.storage)? + 1;
    UNBOND_REPLY_ID.save(deps.storage, &unbond_reply_id)?;
    REPLY_RECEIVERS.save(deps.storage, unbond_reply_id, &receiver)?;
    let submsg: SubMsg<NeutronMsg> = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: config.core_contract,
            msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Unbond {})?,
            funds: vec![coin.clone()],
        },
        unbond_reply_id,
    );
    let attrs = vec![attr("receiver", receiver)];
    Ok(response("execute_unbond", CONTRACT_NAME, attrs).add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    const U32_MAX_AS_U64: u64 = u32::MAX as u64;
    /*
       |         unbond         |                 withdraw                |   ibc_transfer_sudo_reply   |
       | 0 --- ... --- u32::MAX | u32::MAX + 1 --- ... --- (u64::MAX - 1) |           u64::MAX          |
       <-------------------------------------- u64::MAX ----------------------------------------------->
    */
    match msg.id {
        IBC_TRANSFER_SUDO_REPLY_ID => store_seq_id(deps, env, msg),
        0..=U32_MAX_AS_U64 => finalize_unbond(deps, env, msg),
        _ => finalize_withdraw(deps, env, msg),
    }
}

pub fn store_seq_id(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
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
    let coin = REPLY_TRANSFER_COIN.load(deps.storage)?;
    SUDO_SEQ_ID_TO_COIN.save(deps.storage, seq_id, &coin)?;
    Ok(response(
        "reply_store_seq_id",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

pub fn finalize_withdraw(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(res) => {
            let Config {
                source_channel,
                source_port,
                ibc_timeout,
                ..
            } = CONFIG.load(deps.storage)?;
            let withdraw_reply_id: u64 = msg.id;
            let receiver = REPLY_RECEIVERS.load(deps.storage, withdraw_reply_id)?;
            let transfer_event = res
                .events
                .iter()
                .find(|x| x.ty == "transfer")
                .ok_or(ContractError::NoTransferEvent)?;
            let coin_attr = Coin::from_str(
                &transfer_event
                    .attributes
                    .iter()
                    .find(|x| x.key == "amount")
                    .ok_or(ContractError::NoTransferAmountFound)?
                    .value,
            )?;
            let ibc_send_submsg = SubMsg::reply_on_success(
                NeutronMsg::IbcTransfer {
                    source_port: source_port.clone(),
                    source_channel: source_channel.clone(),
                    token: coin_attr.clone(),
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
            let attrs = vec![
                attr("source_port", source_port),
                attr("source_channel", source_channel),
                attr("receiver", receiver),
                attr(
                    "timeout",
                    env.block.time.plus_seconds(ibc_timeout).nanos().to_string(),
                ),
                attr("amount", coin_attr.to_string()),
            ];
            REPLY_TRANSFER_COIN.save(deps.storage, &coin_attr)?;
            Ok(response("reply_finalize_withdraw", CONTRACT_NAME, attrs)
                .add_submessages(vec![ibc_send_submsg]))
        }
        cosmwasm_std::SubMsgResult::Err(_) => unreachable!(),
    }
}

pub fn finalize_unbond(
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
            let unbond_reply_id: u64 = msg.id;
            let receiver = REPLY_RECEIVERS.load(deps.storage, unbond_reply_id)?;
            let nft_mint_event = res
                .events
                .iter()
                .find(|x| x.ty == "wasm")
                .ok_or(ContractError::NoNFTMinted)?;
            let nft_name = &nft_mint_event
                .attributes
                .iter()
                .find(|x| x.key == "token_id")
                .ok_or(ContractError::NoNFTMintedFound)?
                .value;
            let (batch, unbond_id) = parse_nft(nft_name.clone())?;
            let tf_token_subdenom = format!("nft_{:?}_{:?}", batch, unbond_id);
            let tf_create_denom_msg: CosmosMsg<NeutronMsg> =
                CosmosMsg::Custom(NeutronMsg::CreateDenom {
                    subdenom: tf_token_subdenom.clone(),
                });
            // query_full_denom doesn't work here because denom hasn't been produced yet
            let full_tf_denom = format!("factory/{}/{}", env.contract.address, tf_token_subdenom);
            let tf_mint_voucher_msg: CosmosMsg<NeutronMsg> =
                CosmosMsg::Custom(NeutronMsg::MintTokens {
                    denom: full_tf_denom.clone(),
                    amount: Uint128::from(1u128),
                    mint_to_address: env.contract.address.to_string(),
                });
            let coin = Coin {
                denom: full_tf_denom.clone(),
                amount: Uint128::from(1u128),
            };
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
            let attrs = vec![
                attr("action", "reply_finalize_bond"),
                attr("reply_id", unbond_reply_id.to_string()),
                attr("nft", nft_name),
                attr("to_address", receiver.clone()),
                attr("source_port", source_port.to_string()),
                attr("source_channel", source_channel.clone()),
                attr("ibc_timeout", ibc_timeout.to_string()),
                attr("tf_denom", full_tf_denom.clone()),
            ];
            REPLY_TRANSFER_COIN.save(deps.storage, &coin)?;
            TF_DENOM_TO_NFT_ID.save(deps.storage, full_tf_denom, nft_name)?;
            REPLY_RECEIVERS.remove(deps.storage, unbond_reply_id);
            Ok(response("reply_finalize_unbond", CONTRACT_NAME, attrs)
                .add_messages(vec![tf_create_denom_msg, tf_mint_voucher_msg])
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
        TransferSudoMsg::Response { .. } => sudo_response(),
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
    let actual_denom = SUDO_SEQ_ID_TO_COIN
        .load(deps.storage, req.sequence.unwrap())?
        .denom;
    let packet: FungibleTokenPacketData = from_json(req.data.unwrap())?;
    let packet_amount = Uint128::from_str(packet.amount.as_str())?;

    // If given ibc-transfer for given receiver on the remote chain fails then
    // current contract owns these tokens right now. Memorize in the map, that
    // for given user our contract possess these failed-to-process tokens
    FAILED_TRANSFERS.update(
        deps.storage,
        packet.receiver,
        |current_debt| match current_debt {
            Some(funds) => Ok::<Vec<Coin>, ContractError>(
                // If given denom exist - just modify its amount
                if funds.iter().any(|coin| coin.denom == actual_denom) {
                    funds
                        .iter()
                        .map(|coin| {
                            if coin.denom == actual_denom {
                                Coin {
                                    denom: coin.denom.clone(),
                                    amount: coin.amount + packet_amount,
                                }
                            } else {
                                coin.clone()
                            }
                        })
                        .collect()
                // If it doesn't exist - push it at the end
                } else {
                    funds
                        .iter()
                        .cloned()
                        .chain(std::iter::once(Coin {
                            denom: actual_denom.clone(),
                            amount: packet_amount,
                        }))
                        .collect()
                },
            ),
            None => Ok(vec![Coin {
                denom: actual_denom,
                amount: packet_amount,
            }]), // if receiver doesn't have any pending coins yet
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

fn parse_nft(nft_id: String) -> Result<(u64, u64), ContractError> {
    let parts: Vec<&str> = nft_id.split('_').collect(); // example -- 2_neutron1tpanc442f5u0acajw0rs78yvj5weqpq9q6lvwl_2804

    if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
        if let (Ok(first_num), Ok(last_num)) = (first.parse::<u64>(), last.parse::<u64>()) {
            return Ok((first_num, last_num));
        } else {
            return Err(ContractError::NFTParseError {});
        }
    }
    Err(ContractError::NFTParseError {})
}
