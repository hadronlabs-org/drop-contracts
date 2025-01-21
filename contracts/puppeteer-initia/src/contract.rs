use cosmos_sdk_proto::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin};
use cosmos_sdk_proto::{
    cosmos::{authz::v1beta1::MsgExec, distribution::v1beta1::MsgSetWithdrawAddress},
    traits::MessageExt,
};
use cosmwasm_std::{
    attr, ensure, to_json_binary, Addr, Attribute, CosmosMsg, Deps, Order, Reply, StdError, SubMsg,
    Timestamp, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use drop_helpers::{
    answer::response, ibc_client_state::query_client_state, ibc_fee::query_ibc_fee,
    icq_initia::new_delegations_and_balance_query_msg, interchain::prepare_any_msg,
    validation::validate_addresses,
};
use drop_proto::proto::initia::mstaking::v1::InitiaMsgDelegate;
use drop_proto::proto::{
    cosmos::base::v1beta1::Coin as ProtoCoin, initia::mstaking::v1::MsgBeginRedelegate,
    liquidstaking::distribution::v1beta1::MsgWithdrawDelegatorReward,
};
use drop_puppeteer_base::{
    error::{ContractError, ContractResult},
    msg::{QueryMsg, TransferReadyBatchesMsg},
    peripheral_hook::{
        ReceiverExecuteMsg, ResponseHookErrorMsg, ResponseHookMsg, ResponseHookSuccessMsg,
        Transaction,
    },
    r#trait::PuppeteerReconstruct,
    state::{
        BalancesAndDelegationsState, PuppeteerBase, ReplyMsg, TxState, TxStateStatus, ICA_ID,
        LOCAL_DENOM,
    },
};
use drop_staking_base::{
    msg::puppeteer::{
        BalancesResponse, DelegationsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryExtMsg,
    },
    state::{
        puppeteer::{Config, ConfigOptional, Delegations, KVQueryType},
        puppeteer_initia::BalancesAndDelegations,
    },
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery, types::ProtobufAny},
    interchain_queries::{queries::get_raw_interchain_query_result, v045::types::Balances},
    sudo::msg::{RequestPacket, SudoMsg},
    NeutronResult,
};
use std::vec;

pub type Puppeteer<'a> = PuppeteerBase<'a, Config, KVQueryType, BalancesAndDelegations>;

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
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
    validate_timeout(msg.timeout)?;
    if !msg.remote_denom.starts_with("move/") {
        return Err(ContractError::InvalidRemoteDenom);
    }
    let config = &Config {
        connection_id: msg.connection_id,
        port_id: msg.port_id,
        update_period: msg.update_period,
        remote_denom: msg.remote_denom,
        allowed_senders,
        transfer_channel_id: msg.transfer_channel_id,
        sdk_version: msg.sdk_version,
        timeout: msg.timeout,
        native_bond_provider: deps.api.addr_validate(&msg.native_bond_provider)?,
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
            QueryExtMsg::NonNativeRewardsBalances {} => unimplemented!(),
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

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
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
        ExecuteMsg::TokenizeShare { .. } => unimplemented!(),
        ExecuteMsg::RedeemShares { .. } => {
            unimplemented!()
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
        ExecuteMsg::RegisterDelegatorUnbondingDelegationsQuery { .. } => {
            unimplemented!()
        }
        ExecuteMsg::RegisterNonNativeRewardsBalancesQuery { .. } => unimplemented!(),
        ExecuteMsg::Transfer { items, reply_to } => execute_transfer(deps, info, items, reply_to),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            let attrs = vec![attr("action", "update_ownership")];
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response("update_ownership", CONTRACT_NAME, attrs))
        }
        ExecuteMsg::SetupProtocol {
            rewards_withdraw_address,
        } => execute_setup_protocol(deps, env, info, rewards_withdraw_address),
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
        if !remote_denom.starts_with("move/") {
            return Err(ContractError::InvalidRemoteDenom);
        }
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

fn execute_delegate(
    mut deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;

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
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
        attr("amount_to_stake", amount_to_stake.to_string()),
    ];
    let ica_address = puppeteer_base.ica.get_address(deps.storage)?;

    /*

        fn get_delegate_msg(
        chain_type: ChainType,
        delegator: String,
        validator: String,
        denom: String,
        amount: String,
    ) -> ContractResult<cosmos_sdk_proto::Any> {
        match chain_type {
            ChainType::BasicCosmos => Ok(cosmos_sdk_proto::Any {
                type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                value: MsgDelegate {
                    delegator_address: delegator,
                    validator_address: validator,
                    amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin { denom, amount }),
                }
                .to_bytes()?,
            }),
            ChainType::Initia => Ok(cosmos_sdk_proto::Any {
                type_url: "/initia.mstaking.v1.MsgDelegate".to_string(),
                value: InitiaMsgDelegate {
                    delegator_address: delegator,
                    validator_address: validator,
                    amount: vec![drop_proto::proto::cosmos::base::v1beta1::Coin { denom, amount }],
                }
                .to_bytes()?,
            }),
        }
    }

         */

    let mut any_delegation_msgs = vec![];
    for (validator, amount) in items.clone() {
        let delegation = InitiaMsgDelegate {
            delegator_address: ica_address.to_string(),
            validator_address: validator.to_string(),
            amount: vec![drop_proto::proto::cosmos::base::v1beta1::Coin {
                denom: config.remote_denom.to_string(),
                amount: amount.to_string(),
            }],
        };
        any_delegation_msgs.push(prepare_any_msg(
            delegation,
            "/initia.mstaking.v1.MsgDelegate",
        )?);
    }

    let submsg = compose_submsg(
        deps.branch(),
        config,
        any_delegation_msgs,
        Transaction::Stake {
            amount: amount_to_stake,
        },
        reply_to,
        ReplyMsg::SudoPayload.to_reply_id(),
    )?;

    Ok(response("stake", CONTRACT_NAME, attrs).add_submessage(submsg))
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
    deps.api.debug(&format!(
        "WASMDEBUG: register_delegations_and_balance_query validators:{:?}",
        validators
    ));
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
            )?,
            ReplyMsg::KvDelegationsAndBalance { i: i as u16 }.to_reply_id(),
        ));
    }
    Ok(Response::new()
        .add_messages(messages)
        .add_submessages(submessages))
}

fn execute_setup_protocol(
    mut deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    rewards_withdraw_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();
    let config: Config = puppeteer_base.config.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    puppeteer_base.validate_tx_idle_state(deps.as_ref())?;
    let ica = puppeteer_base.ica.get_address(deps.storage)?;
    let mut any_msgs = vec![];

    let set_withdraw_address_msg = MsgSetWithdrawAddress {
        delegator_address: ica.to_string(),
        withdraw_address: rewards_withdraw_address.clone(),
    };

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
            type_url: "/initia.mstaking.v1.MsgUndelegate".to_string(),
            value: drop_proto::proto::initia::mstaking::v1::MsgUndelegate {
                delegator_address: delegator.to_string(),
                validator_address: validator.to_string(),
                amount: vec![drop_proto::proto::cosmos::base::v1beta1::Coin {
                    denom: config.remote_denom.to_string(),
                    amount: amount.to_string(),
                }],
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
        amount: vec![ProtoCoin {
            denom: config.remote_denom.to_string(),
            amount: amount.to_string(),
        }],
    };

    let submsg = compose_submsg(
        deps.branch(),
        config.clone(),
        vec![prepare_any_msg(
            redelegate_msg,
            "/initia.mstaking.v1.MsgBeginRedelegate",
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
                KVQueryType::NonNativeRewardsBalances => unimplemented!(),
                KVQueryType::UnbondingDelegations => unimplemented!(),
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
    _data: Binary,
) -> NeutronResult<Response<NeutronMsg>> {
    deps.api.debug("WASMDEBUG: sudo response");
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
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
                transaction: transaction.clone(),
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
                    transaction: transaction.clone(),
                    local_height: env.block.height,
                    remote_height: remote_height.u64(),
                }),
            ))?,
            funds: vec![],
        }));
    }
    Ok(response("sudo-response", "puppeteer", attrs).add_messages(msgs))
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

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tx_state
            .reply_to
            .ok_or_else(|| StdError::generic_err("reply_to not found"))?,
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
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
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
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
    let puppeteer_base: PuppeteerBase<'_, Config, KVQueryType, BalancesAndDelegations> =
        Puppeteer::default();
    match ReplyMsg::from_reply_id(msg.id) {
        ReplyMsg::SudoPayload => puppeteer_base.submit_tx_reply(deps, msg),
        ReplyMsg::IbcTransfer => puppeteer_base.submit_ibc_transfer_reply(deps, msg),
        ReplyMsg::KvDelegationsAndBalance { i } => {
            deps.api.debug(&format!(
                "WASMDEBUG: DELEGATIONS_AND_BALANCE_REPLY_ID {:?}",
                msg
            ));
            puppeteer_base.register_delegations_and_balance_query_reply(
                deps,
                msg,
                i,
                KVQueryType::DelegationsAndBalance,
            )
        }
        ReplyMsg::KvNonNativeRewardsBalances => unimplemented!(),
        ReplyMsg::KvUnbondingDelegations { .. } => unimplemented!(),
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

fn sudo_delegations_and_balance_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    query_id: u64,
    version: &str,
) -> NeutronResult<Response<NeutronMsg>> {
    let puppeteer_base: PuppeteerBase<'_, Config, KVQueryType, BalancesAndDelegations> =
        Puppeteer::default();
    let chunks_len = puppeteer_base
        .delegations_and_balances_query_id_chunk
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    let chunk_id = puppeteer_base
        .delegations_and_balances_query_id_chunk
        .load(deps.storage, query_id)?;
    let (remote_height, kv_results) = {
        let registered_query_result = get_raw_interchain_query_result(deps.as_ref(), query_id)?;
        (
            registered_query_result.result.height,
            registered_query_result.result.kv_results,
        )
    };
    deps.api.debug(&format!(
        "WASMDEBUG KVQueryResult kv_results: {:?}",
        kv_results
    ));
    let data: BalancesAndDelegations = PuppeteerReconstruct::reconstruct(
        &kv_results,
        version,
        Some(
            puppeteer_base
                .config
                .load(deps.storage)?
                .remote_denom
                .as_str(),
        ),
    )?;
    let new_state = match puppeteer_base
        .delegations_and_balances
        .may_load(deps.storage, &remote_height)?
    {
        Some(mut state) => {
            if !state.collected_chunks.contains(&chunk_id) {
                state
                    .data
                    .delegations
                    .delegations
                    .extend(data.delegations.delegations);
                state.collected_chunks.push(chunk_id);
            }
            state
        }
        None => BalancesAndDelegationsState {
            data,
            remote_height,
            local_height: env.block.height,
            timestamp: env.block.time,
            collected_chunks: vec![chunk_id],
        },
    };
    if new_state.collected_chunks.len() == chunks_len {
        let prev_key = puppeteer_base
            .last_complete_delegations_and_balances_key
            .load(deps.storage)
            .unwrap_or_default();
        if prev_key < remote_height {
            puppeteer_base
                .last_complete_delegations_and_balances_key
                .save(deps.storage, &remote_height)?;
        }
    }
    puppeteer_base
        .delegations_and_balances
        .save(deps.storage, &remote_height, &new_state)?;
    Ok(Response::default())
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
