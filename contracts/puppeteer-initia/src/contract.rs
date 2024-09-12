use cosmos_sdk_proto::cosmos::{
    authz::v1beta1::{GenericAuthorization, Grant, MsgGrant, MsgGrantResponse},
    bank::v1beta1::{MsgSend, MsgSendResponse},
    base::{abci::v1beta1::TxMsgData, v1beta1::Coin},
};
use cosmos_sdk_proto::{
    cosmos::{authz::v1beta1::MsgExec, distribution::v1beta1::MsgSetWithdrawAddress},
    traits::MessageExt,
};
use cosmwasm_std::{
    attr, ensure_eq, to_json_binary, Addr, Attribute, CosmosMsg, Deps, Order, Reply, StdError,
    SubMsg, Timestamp, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use drop_helpers::{
    answer::response,
    ibc_client_state::query_client_state,
    ibc_fee::query_ibc_fee,
    icq::{
        new_delegations_and_balance_query_msg, new_multiple_balances_query_msg,
        update_multiple_balances_query_msg,
    },
    interchain::prepare_any_msg,
    validation::validate_addresses,
};
use drop_proto::proto::{
    cosmos::base::v1beta1::Coin as ProtoCoin,
    liquidstaking::{
        distribution::v1beta1::MsgWithdrawDelegatorReward,
        staking::v1beta1::{
            MsgBeginRedelegate, MsgBeginRedelegateResponse, MsgDelegateResponse,
            MsgRedeemTokensforShares, MsgRedeemTokensforSharesResponse, MsgTokenizeShares,
            MsgTokenizeSharesResponse, MsgUndelegateResponse,
        },
    },
};
use drop_puppeteer_base::{
    error::{ContractError, ContractResult},
    msg::{
        IBCTransferReason, QueryMsg, ReceiverExecuteMsg, ResponseAnswer, ResponseHookErrorMsg,
        ResponseHookMsg, ResponseHookSuccessMsg, Transaction, TransferReadyBatchesMsg,
    },
    proto::MsgIBCTransfer,
    state::{
        Delegations, PuppeteerBase, RedeemShareItem, ReplyMsg, TxState, TxStateStatus,
        UnbondingDelegation, ICA_ID, LOCAL_DENOM,
    },
};
use drop_staking_base::{
    msg::puppeteer::{
        BalancesResponse, DelegationsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryExtMsg,
    },
    state::puppeteer::{Config, ConfigOptional, KVQueryType, NON_NATIVE_REWARD_BALANCES},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery, types::ProtobufAny},
    interchain_queries::v045::{
        new_register_delegator_unbonding_delegations_query_msg, types::Balances,
    },
    interchain_txs::helpers::decode_message_response,
    sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, SudoMsg},
    NeutronResult,
};
use prost::Message;
use std::{str::FromStr, vec};

pub type Puppeteer<'a> = PuppeteerBase<'a, Config, KVQueryType>;

const CONTRACT_NAME: &str = concat!("crates.io:drop-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_DELEGATIONS_QUERIES_CHUNK_SIZE: u32 = 15;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let allowed_senders = validate_addresses(
        deps.as_ref().into_empty(),
        msg.allowed_senders.as_ref(),
        None,
    )?;
    let owner = deps
        .api
        .addr_validate(&msg.owner.unwrap_or(info.sender.to_string()))?
        .to_string();
    validate_timeout(msg.timeout)?;
    let config = &Config {
        connection_id: msg.connection_id,
        port_id: msg.port_id,
        update_period: msg.update_period,
        remote_denom: msg.remote_denom,
        allowed_senders,
        transfer_channel_id: msg.transfer_channel_id,
        sdk_version: msg.sdk_version,
        timeout: msg.timeout,
        delegations_queries_chunk_size: msg
            .delegations_queries_chunk_size
            .unwrap_or(DEFAULT_DELEGATIONS_QUERIES_CHUNK_SIZE),
    };
    Puppeteer::default().instantiate(deps, config, owner)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(
    deps: Deps<NeutronQuery>,
    env: Env,
    msg: QueryMsg<QueryExtMsg>,
) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Extension { msg } => match msg {
            QueryExtMsg::Delegations {} => query_delegations(deps),
            QueryExtMsg::Balances {} => query_balances(deps),
            QueryExtMsg::NonNativeRewardsBalances {} => query_non_native_rewards_balances(deps),
            QueryExtMsg::UnbondingDelegations {} => to_json_binary(
                &Puppeteer::default()
                    .unbonding_delegations
                    .range(deps.storage, None, None, Order::Ascending)
                    .map(|res| res.map(|(_key, value)| value))
                    .collect::<StdResult<Vec<_>>>()?,
            )
            .map_err(ContractError::Std),
            QueryExtMsg::Ownership {} => {
                let owner = cw_ownable::get_ownership(deps.storage)?;
                to_json_binary(&owner).map_err(ContractError::Std)
            }
        },
        QueryMsg::KVQueryIds {} => query_kv_query_ids(deps),
        _ => Puppeteer::default().query(deps, env, msg),
    }
}

fn query_kv_query_ids(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let kv_query_ids: StdResult<Vec<(_, _)>> = Puppeteer::default()
        .kv_queries
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    Ok(to_json_binary(&kv_query_ids?)?)
}

fn query_delegations(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let puppeteer_base = Puppeteer::default();
    match puppeteer_base
        .last_complete_delegations_and_balances_key
        .may_load(deps.storage)?
    {
        None => to_json_binary(&DelegationsResponse {
            delegations: Delegations {
                delegations: vec![],
            },
            remote_height: 0,
            local_height: 0,
            timestamp: Timestamp::default(),
        }),
        Some(last_key) => {
            let last_data = puppeteer_base
                .delegations_and_balances
                .load(deps.storage, &last_key)?;
            to_json_binary(&DelegationsResponse {
                delegations: last_data.data.delegations,
                remote_height: last_data.remote_height,
                local_height: last_data.local_height,
                timestamp: last_data.timestamp,
            })
        }
    }
    .map_err(ContractError::Std)
}

fn query_balances(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let puppeteer_base = Puppeteer::default();
    match puppeteer_base
        .last_complete_delegations_and_balances_key
        .may_load(deps.storage)?
    {
        None => to_json_binary(&BalancesResponse {
            balances: Balances { coins: vec![] },
            remote_height: 0,
            local_height: 0,
            timestamp: Timestamp::default(),
        }),
        Some(last_key) => {
            let last_data = puppeteer_base
                .delegations_and_balances
                .load(deps.storage, &last_key)?;
            to_json_binary(&BalancesResponse {
                balances: last_data.data.balances,
                remote_height: last_data.remote_height,
                local_height: last_data.local_height,
                timestamp: last_data.timestamp,
            })
        }
    }
    .map_err(ContractError::Std)
}

fn query_non_native_rewards_balances(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let data = NON_NATIVE_REWARD_BALANCES.load(deps.storage)?;
    to_json_binary(&BalancesResponse {
        balances: Balances {
            coins: data.data.coins,
        },
        remote_height: data.remote_height,
        local_height: data.local_height,
        timestamp: data.timestamp,
    })
    .map_err(ContractError::Std)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    match msg {
        ExecuteMsg::Undelegate {
            items,
            batch_id,
            reply_to,
        } => execute_undelegate(deps, info, items, batch_id, reply_to),
        ExecuteMsg::Redelegate {
            validator_from,
            validator_to,
            amount,
            reply_to,
        } => execute_redelegate(deps, info, validator_from, validator_to, amount, reply_to),
        ExecuteMsg::TokenizeShare {
            validator,
            amount,
            reply_to,
        } => execute_tokenize_share(deps, info, validator, amount, reply_to),
        ExecuteMsg::RedeemShares { items, reply_to } => {
            execute_redeem_shares(deps, info, items, reply_to)
        }
        ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators,
            transfer,
            reply_to,
        } => {
            execute_claim_rewards_and_optionaly_transfer(deps, info, validators, transfer, reply_to)
        }
        ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery { validators } => {
            register_delegations_and_balance_query(deps, info, validators)
        }
        ExecuteMsg::RegisterDelegatorUnbondingDelegationsQuery { validators } => {
            register_unbonding_delegations_query(deps, info, validators)
        }
        ExecuteMsg::RegisterNonNativeRewardsBalancesQuery { denoms } => {
            register_non_native_rewards_balances_query(deps, info, denoms)
        }
        ExecuteMsg::IBCTransfer { reply_to, reason } => {
            execute_ibc_transfer(deps, env, info, reason, reply_to)
        }
        ExecuteMsg::Transfer { items, reply_to } => execute_transfer(deps, info, items, reply_to),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            let attrs = vec![attr("action", "update_ownership")];
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response("update_ownership", CONTRACT_NAME, attrs))
        }
        ExecuteMsg::SetupProtocol {
            delegate_grantee,
            rewards_withdraw_address,
        } => execute_setup_protocol(deps, env, info, delegate_grantee, rewards_withdraw_address),
        _ => puppeteer_base.execute(deps, env, info, msg.to_base_enum()),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let puppeteer_base = Puppeteer::default();
    let mut config = puppeteer_base.config.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(remote_denom) = new_config.remote_denom {
        config.remote_denom = remote_denom.clone();
        attrs.push(attr("remote_denom", remote_denom))
    }

    if let Some(connection_id) = new_config.connection_id {
        config.connection_id = connection_id.clone();
        attrs.push(attr("connection_id", connection_id))
    }

    if let Some(port_id) = new_config.port_id {
        config.port_id = port_id.clone();
        attrs.push(attr("port_id", port_id))
    }

    if let Some(update_period) = new_config.update_period {
        config.update_period = update_period;
        attrs.push(attr("update_period", update_period.to_string()))
    }

    if let Some(allowed_senders) = new_config.allowed_senders {
        let allowed_senders =
            validate_addresses(deps.as_ref().into_empty(), allowed_senders.as_ref(), None)?;
        attrs.push(attr("allowed_senders", allowed_senders.len().to_string()));
        config.allowed_senders = allowed_senders
    }

    if let Some(transfer_channel_id) = new_config.transfer_channel_id {
        config.transfer_channel_id = transfer_channel_id.clone();
        attrs.push(attr("transfer_channel_id", transfer_channel_id))
    }

    if let Some(sdk_version) = new_config.sdk_version {
        config.sdk_version = sdk_version.clone();
        attrs.push(attr("sdk_version", sdk_version))
    }
    if let Some(timeout) = new_config.timeout {
        validate_timeout(timeout)?;
        attrs.push(attr("timeout", timeout.to_string()));
        config.timeout = timeout;
    }

    puppeteer_base.update_config(deps.into_empty(), &config)?;

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn execute_ibc_transfer(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    reason: IBCTransferReason,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    // exclude fees, no need to send local denom tokens to remote zone
    let message_funds: Vec<_> = info
        .funds
        .into_iter()
        .filter(|f| f.denom != LOCAL_DENOM)
        .collect();
    ensure_eq!(
        message_funds.len(),
        1,
        ContractError::InvalidFunds {
            reason: "Only one coin is allowed".to_string()
        }
    );
    let coin = message_funds.first().ok_or(ContractError::InvalidFunds {
        reason: "No funds".to_string(),
    })?;
    let ica_address = puppeteer_base.ica.get_address(deps.storage)?;
    let msg = NeutronMsg::IbcTransfer {
        source_port: config.port_id,
        source_channel: config.transfer_channel_id,
        token: (*coin).clone(),
        sender: env.contract.address.to_string(),
        receiver: ica_address.to_string(),
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: None,
            revision_height: None,
        },
        timeout_timestamp: env.block.time.plus_seconds(config.timeout).nanos(),
        memo: "".to_string(),
        fee: query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?,
    };
    let submsg = puppeteer_base.msg_with_sudo_callback(
        deps,
        msg,
        Transaction::IBCTransfer {
            denom: coin.denom.to_string(),
            amount: coin.amount.into(),
            reason,
            recipient: ica_address,
        },
        reply_to,
        ReplyMsg::IbcTransfer.to_reply_id(),
    )?;
    Ok(Response::default().add_submessages(vec![submsg]))
}

fn register_non_native_rewards_balances_query(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    denoms: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: register_non_native_rewards_balances_query denoms:{:?}",
        denoms
    ));
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let kv_queries = puppeteer_base
        .kv_queries
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<(u64, KVQueryType)>, _>>()?;
    let ica = puppeteer_base.ica.get_address(deps.storage)?;
    let mut messages = vec![];
    let mut submessages = vec![];
    for (query_id, query_type) in kv_queries {
        if query_type == KVQueryType::NonNativeRewardsBalances {
            messages.push(update_multiple_balances_query_msg(
                query_id,
                ica.clone(),
                denoms.clone(),
            )?);
        }
    }
    if messages.is_empty() {
        submessages.push(SubMsg::reply_on_success(
            new_multiple_balances_query_msg(
                config.connection_id.clone(),
                ica.clone(),
                denoms,
                config.update_period,
            )?,
            ReplyMsg::KvNonNativeRewardsBalances.to_reply_id(),
        ));
    }
    deps.api.debug(&format!(
        "WASMDEBUG: register_non_native_rewards_balances_query messages:{:?} submessages:{:?}",
        messages, submessages
    ));
    Ok(Response::new()
        .add_messages(messages)
        .add_submessages(submessages))
}

fn register_delegations_and_balance_query(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    cosmwasm_std::ensure!(
        validators.len() < u16::MAX as usize,
        StdError::generic_err("Too many validators provided")
    );
    let current_queries: Vec<u64> = puppeteer_base
        .delegations_and_balances_query_id_chunk
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let messages = current_queries
        .iter()
        .map(|query_id| {
            puppeteer_base
                .delegations_and_balances_query_id_chunk
                .remove(deps.storage, *query_id);
            puppeteer_base.kv_queries.remove(deps.storage, *query_id);
            NeutronMsg::remove_interchain_query(*query_id)
        })
        .collect::<Vec<_>>();

    let mut submessages = vec![];
    let ica = puppeteer_base.ica.get_address(deps.storage)?;

    for (i, chunk) in validators
        .chunks(config.delegations_queries_chunk_size as usize)
        .enumerate()
    {
        submessages.push(SubMsg::reply_on_success(
            new_delegations_and_balance_query_msg(
                config.connection_id.clone(),
                ica.clone(),
                config.remote_denom.clone(),
                chunk.to_vec(),
                config.update_period,
                config.sdk_version.as_str(),
            )?,
            ReplyMsg::KvDelegationsAndBalance { i: i as u16 }.to_reply_id(),
        ));
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_submessages(submessages))
}

fn register_unbonding_delegations_query(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    cosmwasm_std::ensure!(
        validators.len() < u16::MAX as usize,
        StdError::generic_err("Too many validators provided")
    );

    // TODO: this code will leave behind many registered ICQs when called again
    //       we need to call RegisterDelegations and RegisterUnbondingDelegations together
    //       and update existing queries

    let delegator = puppeteer_base.ica.get_address(deps.storage)?;
    let msgs = validators
        .into_iter()
        .enumerate()
        .map(|(i, validator)| {
            puppeteer_base.unbonding_delegations_reply_id_storage.save(
                deps.storage,
                i as u16,
                &UnbondingDelegation {
                    validator_address: validator.clone(),
                    query_id: 0,
                    unbonding_delegations: vec![],
                    last_updated_height: 0,
                },
            )?;

            Ok(SubMsg::reply_on_success(
                new_register_delegator_unbonding_delegations_query_msg(
                    config.connection_id.clone(),
                    delegator.clone(),
                    vec![validator],
                    config.update_period,
                )?,
                ReplyMsg::KvUnbondingDelegations {
                    validator_index: i as u16,
                }
                .to_reply_id(),
            ))
        })
        .collect::<ContractResult<Vec<_>>>()?;

    Ok(Response::new().add_submessages(msgs))
}

fn execute_setup_protocol(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    delegate_grantee: String,
    rewards_withdraw_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let ica = puppeteer_base.ica.get_address(deps.storage)?;
    let mut any_msgs = vec![];
    let grant_msg = MsgGrant {
        grantee: delegate_grantee.clone(),
        granter: ica.to_string(),
        grant: Some(Grant {
            authorization: Some(cosmos_sdk_proto::Any {
                type_url: "/cosmos.authz.v1beta1.GenericAuthorization".to_string(),
                value: GenericAuthorization {
                    msg: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                }
                .encode_to_vec(),
            }),
            expiration: Some(prost_types::Timestamp {
                seconds: env
                    .block
                    .time
                    .plus_days(365 * 120 + 30)
                    .seconds()
                    .try_into()
                    .map_err(|_| ContractError::Std(StdError::generic_err("Invalid timestamp")))?,
                nanos: 0,
            }),
        }),
    };
    let set_withdraw_address_msg = MsgSetWithdrawAddress {
        delegator_address: ica.to_string(),
        withdraw_address: rewards_withdraw_address.clone(),
    };
    any_msgs.push(prepare_any_msg(
        grant_msg,
        "/cosmos.authz.v1beta1.MsgGrant",
    )?);
    any_msgs.push(prepare_any_msg(
        set_withdraw_address_msg,
        "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress",
    )?);
    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        any_msgs,
        Transaction::SetupProtocol {
            interchain_account_id: ica.to_string(),
            delegate_grantee,
            rewards_withdraw_address,
        },
        "".to_string(),
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_transfer(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    items: Vec<(String, cosmwasm_std::Coin)>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let ica = puppeteer_base.ica.get_address(deps.storage)?;
    let mut any_msgs = vec![];
    for (val, amount) in items.clone() {
        let transfer_msg = MsgSend {
            from_address: ica.to_string(),
            to_address: val.to_string(),
            amount: vec![Coin {
                amount: amount.amount.to_string(),
                denom: amount.denom,
            }],
        };
        deps.api.debug(&format!(
            "WASMDEBUG: transfer msg: {:?} to: {:?}",
            transfer_msg, val
        ));
        any_msgs.push(prepare_any_msg(
            transfer_msg,
            "/cosmos.bank.v1beta1.MsgSend",
        )?);
    }
    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        any_msgs,
        Transaction::Transfer {
            interchain_account_id: ICA_ID.to_string(),
            items,
        },
        reply_to,
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_claim_rewards_and_optionaly_transfer(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<String>,
    transfer: Option<TransferReadyBatchesMsg>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let ica = puppeteer_base.ica.get_address(deps.storage)?;
    let mut any_msgs = vec![];
    if let Some(transfer) = transfer.clone() {
        let transfer_msg = MsgSend {
            from_address: ica.to_string(),
            to_address: transfer.recipient,
            amount: vec![Coin {
                amount: transfer.amount.to_string(),
                denom: config.remote_denom.to_string(),
            }],
        };
        any_msgs.push(prepare_any_msg(
            transfer_msg,
            "/cosmos.bank.v1beta1.MsgSend",
        )?);
    }

    let mut claim_msgs = vec![];
    for val in validators.clone() {
        claim_msgs.push(cosmos_sdk_proto::Any {
            type_url: "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward".to_string(),
            value: MsgWithdrawDelegatorReward {
                delegator_address: ica.to_string(),
                validator_address: val,
            }
            .to_bytes()?,
        })
    }

    let grant_msg = MsgExec {
        grantee: ica.to_string(),
        msgs: claim_msgs,
    };

    any_msgs.push(prepare_any_msg(grant_msg, "/cosmos.authz.v1beta1.MsgExec")?);

    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        any_msgs,
        Transaction::ClaimRewardsAndOptionalyTransfer {
            interchain_account_id: ICA_ID.to_string(),
            validators,
            denom: config.remote_denom.to_string(),
            transfer,
        },
        reply_to,
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_undelegate(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
    batch_id: u128,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let delegator = puppeteer_base.ica.get_address(deps.storage)?;
    let mut undelegation_msgs = vec![];
    for (validator, amount) in items.iter() {
        undelegation_msgs.push(cosmos_sdk_proto::Any {
            type_url: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
            value: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate {
                delegator_address: delegator.to_string(),
                validator_address: validator.to_string(),
                amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                    denom: config.remote_denom.to_string(),
                    amount: amount.to_string(),
                }),
            }
            .to_bytes()?,
        })
    }

    let grant_msg = MsgExec {
        grantee: delegator,
        msgs: undelegation_msgs,
    };

    let any_msgs: Vec<neutron_sdk::bindings::types::ProtobufAny> =
        vec![prepare_any_msg(grant_msg, "/cosmos.authz.v1beta1.MsgExec")?];

    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        any_msgs,
        Transaction::Undelegate {
            interchain_account_id: ICA_ID.to_string(),
            denom: config.remote_denom,
            batch_id,
            items,
        },
        reply_to,
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_redelegate(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validator_from: String,
    validator_to: String,
    amount: Uint128,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let delegator = puppeteer_base.ica.get_address(deps.storage)?;
    let redelegate_msg = MsgBeginRedelegate {
        delegator_address: delegator,
        validator_src_address: validator_from.to_string(),
        validator_dst_address: validator_to.to_string(),
        amount: Some(ProtoCoin {
            denom: config.remote_denom.to_string(),
            amount: amount.to_string(),
        }),
    };

    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        vec![prepare_any_msg(
            redelegate_msg,
            "/cosmos.staking.v1beta1.MsgBeginRedelegate",
        )?],
        Transaction::Redelegate {
            interchain_account_id: ICA_ID.to_string(),
            validator_from,
            validator_to,
            denom: config.remote_denom,
            amount: amount.into(),
        },
        reply_to,
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_tokenize_share(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validator: String,
    amount: Uint128,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let delegator = puppeteer_base.ica.get_address(deps.storage)?;
    let tokenize_msg = MsgTokenizeShares {
        delegator_address: delegator.clone(),
        validator_address: validator.to_string(),
        tokenized_share_owner: delegator,
        amount: Some(ProtoCoin {
            denom: config.remote_denom.to_string(),
            amount: amount.to_string(),
        }),
    };
    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        vec![prepare_any_msg(
            tokenize_msg,
            "/cosmos.staking.v1beta1.MsgTokenizeShares",
        )?],
        Transaction::TokenizeShare {
            interchain_account_id: ICA_ID.to_string(),
            validator,
            denom: config.remote_denom,
            amount: amount.into(),
        },
        reply_to,
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_redeem_shares(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    items: Vec<RedeemShareItem>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![
        attr("action", "redeem_share"),
        attr("items", format!("{:?}", items)),
    ];
    let puppeteer_base = Puppeteer::default();
    deps.api.addr_validate(&reply_to)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    let delegator = puppeteer_base.ica.get_address(deps.storage)?;
    let any_msgs = items
        .iter()
        .map(|one| MsgRedeemTokensforShares {
            delegator_address: delegator.to_string(),
            amount: Some(ProtoCoin {
                denom: one.remote_denom.to_string(),
                amount: one.amount.to_string(),
            }),
        })
        .map(|msg| prepare_any_msg(msg, "/cosmos.staking.v1beta1.MsgRedeemTokensForShares"))
        .collect::<NeutronResult<Vec<ProtobufAny>>>()?;
    let submsg = compose_submsg(
        deps.branch(),
        config,
        any_msgs,
        Transaction::RedeemShares {
            interchain_account_id: ICA_ID.to_string(),
            items,
        },
        reply_to,
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;
    Ok(Response::default()
        .add_submessages(vec![submsg])
        .add_attributes(attrs))
}

fn compose_submsg(
    mut deps: DepsMut<NeutronQuery>,
    config: Config,
    any_msgs: Vec<ProtobufAny>,
    transaction: Transaction,
    reply_to: String,
    reply_id: u64,
) -> NeutronResult<SubMsg<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let ibc_fee = query_ibc_fee(deps.as_ref(), LOCAL_DENOM)?;
    let connection_id = config.connection_id;
    let cosmos_msg = NeutronMsg::submit_tx(
        connection_id,
        ICA_ID.to_string(),
        any_msgs,
        "".to_string(),
        config.timeout,
        ibc_fee,
    );
    let submsg = puppeteer_base.msg_with_sudo_callback(
        deps.branch(),
        cosmos_msg,
        transaction,
        reply_to,
        reply_id,
    )?;
    Ok(submsg)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: SudoMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?} block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_timeout(deps, env, request),
        SudoMsg::TxQueryResult {
            query_id,
            height,
            data,
        } => puppeteer_base.sudo_tx_query_result(deps, env, query_id, height, data),
        SudoMsg::KVQueryResult { query_id } => {
            let query_type = puppeteer_base.kv_queries.load(deps.storage, query_id)?;
            let config = puppeteer_base.config.load(deps.storage)?;
            deps.api
                .debug(&format!("WASMDEBUG: KVQueryResult type {:?}", query_type));
            match query_type {
                KVQueryType::DelegationsAndBalance => puppeteer_base
                    .sudo_delegations_and_balance_kv_query_result(
                        deps,
                        env,
                        query_id,
                        &config.sdk_version,
                    ),
                KVQueryType::NonNativeRewardsBalances => puppeteer_base.sudo_kv_query_result(
                    deps,
                    env,
                    query_id,
                    &config.sdk_version,
                    NON_NATIVE_REWARD_BALANCES,
                ),
                KVQueryType::UnbondingDelegations => {
                    puppeteer_base.sudo_unbonding_delegations_kv_query_result(deps, env, query_id)
                }
            }
        }
        SudoMsg::OpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => puppeteer_base.sudo_open_ack(
            deps,
            env,
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        ),
    }
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    request: RequestPacket,
    data: Binary,
) -> NeutronResult<Response<NeutronMsg>> {
    deps.api.debug("WASMDEBUG: sudo response");
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let puppeteer_base = Puppeteer::default();
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let channel_id = request
        .clone()
        .source_channel
        .ok_or_else(|| StdError::generic_err("source_channel not found"))?;
    let port_id = request
        .clone()
        .source_port
        .ok_or_else(|| StdError::generic_err("source_port not found"))?;
    let tx_state = puppeteer_base.tx_state.load(deps.storage)?;
    puppeteer_base.validate_tx_waiting_state(deps.as_ref())?;
    let reply_to = tx_state
        .reply_to
        .ok_or_else(|| StdError::generic_err("reply_to not found"))?;
    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    deps.api.debug(&format!(
        "WASMDEBUG: transaction: {transaction:?}",
        transaction = transaction
    ));
    puppeteer_base.tx_state.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::Idle,
            seq_id: None,
            transaction: None,
            reply_to: None,
        },
    )?;
    let answers = match transaction {
        Transaction::IBCTransfer { .. } => vec![ResponseAnswer::IBCTransfer(MsgIBCTransfer {})],
        _ => {
            let msg_data: TxMsgData = TxMsgData::decode(data.as_slice())?;
            get_answers_from_msg_data(deps.as_ref(), msg_data)?
        }
    };

    let client_state = query_client_state(&deps.as_ref(), channel_id, port_id)?;
    let remote_height = client_state
        .identified_client_state
        .ok_or_else(|| StdError::generic_err("IBC client state identified_client_state not found"))?
        .client_state
        .latest_height
        .ok_or_else(|| StdError::generic_err("IBC client state latest_height not found"))?
        .revision_height;

    deps.api.debug(&format!(
        "WASMDEBUG: json: {request:?}",
        request = to_json_binary(&ReceiverExecuteMsg::PuppeteerHook(
            ResponseHookMsg::Success(ResponseHookSuccessMsg {
                request_id: seq_id,
                request: request.clone(),
                transaction: transaction.clone(),
                answers: answers.clone(),
                local_height: env.block.height,
                remote_height: remote_height.u64(),
            },)
        ))?
    ));
    let mut msgs = vec![];
    if !reply_to.is_empty() {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to.clone(),
            msg: to_json_binary(&ReceiverExecuteMsg::PuppeteerHook(
                ResponseHookMsg::Success(ResponseHookSuccessMsg {
                    request_id: seq_id,
                    request: request.clone(),
                    transaction: transaction.clone(),
                    answers,
                    local_height: env.block.height,
                    remote_height: remote_height.u64(),
                }),
            ))?,
            funds: vec![],
        }));
    }
    Ok(response("sudo-response", "puppeteer", attrs).add_messages(msgs))
}

fn get_answers_from_msg_data(
    deps: Deps<NeutronQuery>,
    msg_data: TxMsgData,
) -> NeutronResult<Vec<ResponseAnswer>> {
    let mut answers = vec![];
    #[allow(deprecated)]
    for item in msg_data.data {
        let answer = match item.msg_type.as_str() {
            "/cosmos.staking.v1beta1.MsgDelegate" => {
                let _out: MsgDelegateResponse = decode_message_response(&item.data)?;
                ResponseAnswer::DelegateResponse(drop_puppeteer_base::proto::MsgDelegateResponse {})
            }
            "/cosmos.staking.v1beta1.MsgUndelegate" => {
                let out: MsgUndelegateResponse = decode_message_response(&item.data)?;
                ResponseAnswer::UndelegateResponse(
                    drop_puppeteer_base::proto::MsgUndelegateResponse {
                        completion_time: out.completion_time.map(|t| t.into()),
                    },
                )
            }
            "/cosmos.staking.v1beta1.MsgTokenizeShares" => {
                let out: MsgTokenizeSharesResponse = decode_message_response(&item.data)?;
                ResponseAnswer::TokenizeSharesResponse(
                    drop_puppeteer_base::proto::MsgTokenizeSharesResponse {
                        amount: out.amount.map(convert_coin).transpose()?,
                    },
                )
            }
            "/cosmos.staking.v1beta1.MsgBeginRedelegate" => {
                let out: MsgBeginRedelegateResponse = decode_message_response(&item.data)?;
                ResponseAnswer::BeginRedelegateResponse(
                    drop_puppeteer_base::proto::MsgBeginRedelegateResponse {
                        completion_time: out.completion_time.map(|t| t.into()),
                    },
                )
            }
            "/cosmos.authz.v1beta1.MsgGrant" => {
                let _out: MsgGrantResponse = decode_message_response(&item.data)?;
                ResponseAnswer::GrantDelegateResponse(
                    drop_puppeteer_base::proto::MsgGrantResponse {},
                )
            }
            "/cosmos.staking.v1beta1.MsgRedeemTokensForShares" => {
                let out: MsgRedeemTokensforSharesResponse = decode_message_response(&item.data)?;
                ResponseAnswer::RedeemTokensforSharesResponse(
                    drop_puppeteer_base::proto::MsgRedeemTokensforSharesResponse {
                        amount: out.amount.map(convert_coin).transpose()?,
                    },
                )
            }
            "/cosmos.bank.v1beta1.MsgSend" => {
                let _out: MsgSendResponse = decode_message_response(&item.data)?;
                ResponseAnswer::TransferResponse(drop_puppeteer_base::proto::MsgSendResponse {})
            }
            _ => {
                deps.api.debug(
                    format!("This type of acknowledgement is not implemented: {item:?}").as_str(),
                );
                ResponseAnswer::UnknownResponse {}
            }
        };
        deps.api
            .debug(&format!("WASMDEBUG: sudo_response: answer: {answer:?}",));
        answers.push(answer);
    }
    Ok(answers)
}

fn convert_coin(
    coin: drop_proto::proto::cosmos::base::v1beta1::Coin,
) -> StdResult<cosmwasm_std::Coin> {
    Ok(cosmwasm_std::Coin {
        denom: coin.denom,
        amount: Uint128::from_str(&coin.amount)?,
    })
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    details: String,
) -> NeutronResult<Response<NeutronMsg>> {
    let attrs = vec![
        attr("action", "sudo_error"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
        attr("details", details.clone()),
    ];
    let puppeteer_base: PuppeteerBase<'_, Config, KVQueryType> = Puppeteer::default();
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_error: request: {request:?} details: {details:?}",
    ));
    let tx_state = puppeteer_base.tx_state.load(deps.storage)?;
    puppeteer_base.validate_tx_waiting_state(deps.as_ref())?;

    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tx_state
            .reply_to
            .ok_or_else(|| StdError::generic_err("reply_to not found"))?,
        msg: to_json_binary(&ReceiverExecuteMsg::PuppeteerHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
                request_id: seq_id,
                request,
                transaction,
                details,
            },
        )))?,
        funds: vec![],
    });
    puppeteer_base.tx_state.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::Idle,
            seq_id: None,
            transaction: None,
            reply_to: None,
        },
    )?;
    Ok(response("sudo-error", "puppeteer", attrs).add_message(msg))
}

fn sudo_timeout(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
) -> NeutronResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_timeout: request: {request:?}",
        request = request
    ));
    let attrs = vec![
        attr("action", "sudo_timeout"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let puppeteer_base = Puppeteer::default();
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let tx_state = puppeteer_base.tx_state.load(deps.storage)?;
    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    puppeteer_base.validate_tx_waiting_state(deps.as_ref())?;
    puppeteer_base.ica.set_timeout(deps.storage)?;
    puppeteer_base.tx_state.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::Idle,
            seq_id: None,
            transaction: None,
            reply_to: None,
        },
    )?;
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_timeout: request: {request:?}",
        request = request
    ));
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tx_state
            .reply_to
            .ok_or_else(|| StdError::generic_err("reply_to not found"))?,
        msg: to_json_binary(&ReceiverExecuteMsg::PuppeteerHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
                request_id: seq_id,
                request,
                transaction,
                details: "Timeout".to_string(),
            },
        )))?,
        funds: vec![],
    });
    Ok(response("sudo-timeout", "puppeteer", attrs).add_message(msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let puppeteer_base: PuppeteerBase<'_, Config, KVQueryType> = Puppeteer::default();
    match ReplyMsg::from_reply_id(msg.id) {
        ReplyMsg::SudoPayload => puppeteer_base.submit_tx_reply(deps, msg),
        ReplyMsg::IbcTransfer => puppeteer_base.submit_ibc_transfer_reply(deps, msg),
        ReplyMsg::KvDelegationsAndBalance { i } => puppeteer_base
            .register_delegations_and_balance_query_reply(
                deps,
                msg,
                i,
                KVQueryType::DelegationsAndBalance,
            ),
        ReplyMsg::KvNonNativeRewardsBalances => {
            deps.api.debug(&format!(
                "WASMDEBUG: NON_NATIVE_REWARDS_BALANCES_REPLY_ID {:?}",
                msg
            ));
            puppeteer_base.register_kv_query_reply(deps, msg, KVQueryType::NonNativeRewardsBalances)
        }
        ReplyMsg::KvUnbondingDelegations { validator_index } => {
            deps.api.debug(&format!(
                "WASMDEBUG: UNBONDING_DELEGATIONS_REPLY_ID {:?}",
                msg
            ));
            puppeteer_base.register_unbonding_delegations_query_reply(
                deps,
                msg,
                validator_index,
                KVQueryType::UnbondingDelegations,
            )
        }
    }
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

fn validate_sender(config: &Config, sender: &Addr) -> StdResult<()> {
    if config.allowed_senders.contains(sender) {
        Ok(())
    } else {
        Err(StdError::generic_err("Sender is not allowed"))
    }
}

fn validate_timeout(timeout: u64) -> StdResult<()> {
    if timeout < 10 {
        Err(StdError::generic_err(
            "Timeout can not be less than 10 seconds",
        ))
    } else {
        Ok(())
    }
}
