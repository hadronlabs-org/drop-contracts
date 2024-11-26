use cosmos_sdk_proto::cosmos::{
    bank::v1beta1::{MsgSend, MsgSendResponse},
    base::{abci::v1beta1::TxMsgData, v1beta1::Coin},
    distribution::v1beta1::MsgSetWithdrawAddress,
    staking::v1beta1::MsgDelegate,
};
use cosmwasm_std::{
    attr, ensure, to_json_binary, Addr, Attribute, BankMsg, Coin as StdCoin, CosmosMsg, Deps,
    DistributionMsg, Order, Reply, StakingMsg, StdError, SubMsg, Timestamp, Uint128, WasmMsg,
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
    msg::TransferReadyBatchesMsg,
    peripheral_hook::{
        ReceiverExecuteMsg, ResponseAnswer, ResponseHookErrorMsg, ResponseHookMsg,
        ResponseHookSuccessMsg, Transaction,
    },
    proto::MsgIBCTransfer,
    r#trait::PuppeteerReconstruct,
    state::{
        BalancesAndDelegationsState, PuppeteerBase, RedeemShareItem, Transfer, TxState,
        TxStateStatus, UnbondingDelegation, ICA_ID, LOCAL_DENOM,
    },
};
use drop_staking_base::{
    msg::puppeteer_native::{
        BalancesResponse, DelegationsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryExtMsg,
        QueryMsg,
    },
    state::puppeteer_native::{
        reply_msg::ReplyMsg, BalancesAndDelegations, Config, ConfigOptional, Delegations, CONFIG,
        NON_NATIVE_REWARD_BALANCES, RECIPIENT_TRANSFERS,
    },
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery, types::ProtobufAny},
    interchain_queries::{
        queries::get_raw_interchain_query_result,
        v045::{new_register_delegator_unbonding_delegations_query_msg, types::Balances},
    },
    interchain_txs::helpers::decode_message_response,
    sudo::msg::{RequestPacket, SudoMsg},
    NeutronResult,
};
use prost::Message;
use std::{str::FromStr, vec};

const CONTRACT_NAME: &str = concat!("crates.io:drop-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_DELEGATIONS_QUERIES_CHUNK_SIZE: u32 = 15;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
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

    let config = &Config {
        remote_denom: msg.remote_denom,
        allowed_senders,
        native_bond_provider: deps.api.addr_validate(&msg.native_bond_provider)?,
        delegations_queries_chunk_size: msg
            .delegations_queries_chunk_size
            .unwrap_or(DEFAULT_DELEGATIONS_QUERIES_CHUNK_SIZE),
    };

    let attrs: Vec<Attribute> = vec![
        attr("owner", &owner),
        attr("remote_denom", &config.remote_denom),
        attr("allowed_senders", allowed_senders.len().to_string()),
        attr("native_bond_provider", &config.native_bond_provider),
        attr(
            "delegations_queries_chunk_size",
            &config.delegations_queries_chunk_size.to_string(),
        ),
        attr(
            "allowed_senders",
            allowed_senders
                .into_iter()
                .map(|addr| addr.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ),
    ];

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;
    CONFIG.save(deps.storage, config)?;

    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
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
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Transactions {} => query_transactions(deps),
    }
}

fn query_config(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_transactions(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let transfers: Vec<Transfer> = RECIPIENT_TRANSFERS.load(deps.storage)?;
    Ok(to_json_binary(&transfers)?)
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
    match msg {
        ExecuteMsg::Delegate { items, reply_to } => execute_delegate(deps, info, items, reply_to),
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
        ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators,
            transfer,
            reply_to,
        } => {
            execute_claim_rewards_and_optionaly_transfer(deps, info, validators, transfer, reply_to)
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            let attrs = vec![attr("action", "update_ownership")];
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response("update_ownership", CONTRACT_NAME, attrs))
        }
        ExecuteMsg::SetupProtocol {
            rewards_withdraw_address,
        } => execute_setup_protocol(deps, env, info, rewards_withdraw_address),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(remote_denom) = new_config.remote_denom {
        config.remote_denom = remote_denom.clone();
        attrs.push(attr("remote_denom", remote_denom))
    }

    if let Some(allowed_senders) = new_config.allowed_senders {
        let allowed_senders =
            validate_addresses(deps.as_ref().into_empty(), allowed_senders.as_ref(), None)?;
        attrs.push(attr("allowed_senders", allowed_senders.len().to_string()));
        config.allowed_senders = allowed_senders
    }

    if let Some(native_bond_provider) = new_config.native_bond_provider {
        config.native_bond_provider = native_bond_provider.clone();
        attrs.push(attr("native_bond_provider", native_bond_provider))
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn execute_delegate(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;

    let non_staked_balance = deps.querier.query_wasm_smart::<Uint128>(
        &config.native_bond_provider,
        &drop_staking_base::msg::native_bond_provider::QueryMsg::NonStakedBalance {},
    )?;

    ensure!(
        non_staked_balance > Uint128::zero(),
        ContractError::InvalidFunds {
            reason: "no funds to stake".to_string()
        }
    );

    let amount_to_stake = items.iter().map(|(_, amount)| *amount).sum();

    ensure!(
        non_staked_balance >= amount_to_stake,
        ContractError::InvalidFunds {
            reason: "not enough funds to stake".to_string()
        }
    );

    let attrs = vec![
        attr("action", "stake"),
        attr("amount_to_stake", amount_to_stake.to_string()),
    ];

    let mut delegation_msgs = vec![];
    for (validator, amount) in items.clone() {
        let delegate_msg = SubMsg::reply_always(
            StakingMsg::Delegate {
                validator: validator.clone(),
                amount: StdCoin {
                    denom: config.remote_denom.to_string(),
                    amount: amount,
                },
            },
            ReplyMsg::Delegate.to_reply_id(),
        );

        delegation_msgs.push(delegate_msg);
    }

    Ok(response("stake", CONTRACT_NAME, attrs).add_submessages(delegation_msgs))
}

fn execute_setup_protocol(
    mut deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    rewards_withdraw_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    let config: Config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;

    let set_withdraw_address_msg = DistributionMsg::SetWithdrawAddress {
        address: rewards_withdraw_address.clone(),
    };

    Ok(Response::default().add_message(set_withdraw_address_msg))
}

fn execute_claim_rewards_and_optionaly_transfer(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<String>,
    transfer: Option<TransferReadyBatchesMsg>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.addr_validate(&reply_to)?;
    let config: Config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;

    let mut submsgs = vec![];
    if let Some(transfer) = transfer.clone() {
        let send_msg = SubMsg::new(BankMsg::Send {
            to_address: transfer.recipient,
            amount: vec![StdCoin {
                amount: transfer.amount,
                denom: config.remote_denom.to_string(),
            }],
        });

        submsgs.push(send_msg);
    }

    for val in validators.clone() {
        let withdraw_reward_msg =
            SubMsg::new(DistributionMsg::WithdrawDelegatorReward { validator: val });

        submsgs.push(withdraw_reward_msg);
    }

    // let submsg = compose_submsg(
    //     deps.branch(),
    //     config.clone(),
    //     any_msgs,
    //     Transaction::ClaimRewardsAndOptionalyTransfer {
    //         interchain_account_id: ICA_ID.to_string(),
    //         validators,
    //         denom: config.remote_denom.to_string(),
    //         transfer,
    //     },
    //     reply_to,
    //     ReplyMsg::SudoPayload.to_reply_id(),
    // )?;

    Ok(Response::default().add_submessages(submsgs))
}

fn execute_undelegate(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
    batch_id: u128,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.addr_validate(&reply_to)?;
    let config: Config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    let mut undelegation_msgs = vec![];
    for (validator, amount) in items.clone() {
        let delegate_msg = SubMsg::reply_always(
            StakingMsg::Delegate {
                validator: validator.clone(),
                amount: StdCoin {
                    denom: config.remote_denom.to_string(),
                    amount: amount,
                },
            },
            ReplyMsg::Undelegate.to_reply_id(),
        );

        undelegation_msgs.push(delegate_msg);
    }

    Ok(Response::default().add_submessages(undelegation_msgs))
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
                KVQueryType::DelegationsAndBalance => sudo_delegations_and_balance_kv_query_result(
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
    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", seq_id.to_string()),
    ];
    let puppeteer_base = Puppeteer::default();

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
        request = to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
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
            msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
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

pub fn get_answers_from_msg_data(
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
    let puppeteer_base: PuppeteerBase<'_, Config, KVQueryType, BalancesAndDelegations> =
        Puppeteer::default();
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

    let mut fund_to_return = vec![];
    if let Transaction::IBCTransfer { amount, denom, .. } = transaction.clone() {
        fund_to_return.push(StdCoin::new(amount, denom));
    }

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tx_state
            .reply_to
            .ok_or_else(|| StdError::generic_err("reply_to not found"))?,
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
                request_id: seq_id,
                request,
                transaction,
                details,
            },
        )))?,
        funds: fund_to_return,
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
    let mut fund_to_return = vec![];
    if let Transaction::IBCTransfer { amount, denom, .. } = transaction.clone() {
        fund_to_return.push(StdCoin::new(amount, denom));
    }
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
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
                request_id: seq_id,
                request,
                transaction,
                details: "Timeout".to_string(),
            },
        )))?,
        funds: fund_to_return,
    });
    Ok(response("sudo-timeout", "puppeteer", attrs).add_message(msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let puppeteer_base: PuppeteerBase<'_, Config, KVQueryType, BalancesAndDelegations> =
        Puppeteer::default();
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
