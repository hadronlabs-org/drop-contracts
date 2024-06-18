use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, ensure, ensure_eq, ensure_ne, to_json_binary, Addr, Attribute, BankMsg, BankQuery,
    Binary, Coin, CosmosMsg, CustomQuery, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg,
};
use drop_helpers::answer::response;
use drop_helpers::pause::{is_paused, pause_guard, set_pause, unpause, PauseInfoResponse};
use drop_puppeteer_base::msg::{IBCTransferReason, TransferReadyBatchesMsg};
use drop_puppeteer_base::state::RedeemShareItem;
use drop_staking_base::{
    error::core::{ContractError, ContractResult},
    msg::{
        core::{
            ExecuteMsg, InstantiateMsg, LastPuppeteerResponse, LastStakerResponse, MigrateMsg,
            QueryMsg,
        },
        token::{
            ConfigResponse as TokenConfigResponse, ExecuteMsg as TokenExecuteMsg,
            QueryMsg as TokenQueryMsg,
        },
        withdrawal_voucher::ExecuteMsg as VoucherExecuteMsg,
    },
    state::{
        core::{
            unbond_batches_map, Config, ConfigOptional, ContractState, NonNativeRewardsItem,
            UnbondBatch, UnbondBatchStatus, UnbondBatchStatusTimestamps, BONDED_AMOUNT, CONFIG,
            EXCHANGE_RATE, FAILED_BATCH_ID, FSM, LAST_ICA_CHANGE_HEIGHT, LAST_IDLE_CALL,
            LAST_LSM_REDEEM, LAST_PUPPETEER_RESPONSE, LAST_STAKER_RESPONSE, LD_DENOM,
            LSM_SHARES_TO_REDEEM, NON_NATIVE_REWARDS_CONFIG, PENDING_LSM_SHARES, TOTAL_LSM_SHARES,
            UNBOND_BATCH_ID,
        },
        validatorset::ValidatorInfo,
        withdrawal_voucher::{Metadata, Trait},
    },
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use prost::Message;

pub type MessageWithFeeResponse<T> = (CosmosMsg<T>, Option<CosmosMsg<T>>);

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let attrs: Vec<Attribute> = vec![
        attr("token_contract", &msg.token_contract),
        attr("puppeteer_contract", &msg.puppeteer_contract),
        attr("strategy_contract", &msg.strategy_contract),
        attr("base_denom", &msg.base_denom),
        attr("owner", &msg.owner),
    ];
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;
    let config = msg.into_config(deps.as_ref().into_empty())?;
    if let Some(fee) = config.fee {
        if fee < Decimal::zero() || fee > Decimal::one() {
            return Err(ContractError::InvalidFee {});
        }
    }
    CONFIG.save(deps.storage, &config)?;
    LD_DENOM.save(
        deps.storage,
        &deps
            .querier
            .query_wasm_smart::<TokenConfigResponse>(
                &config.token_contract,
                &TokenQueryMsg::Config {},
            )?
            .denom,
    )?;
    //an empty unbonding batch added as it's ready to be used on unbond action
    UNBOND_BATCH_ID.save(deps.storage, &0)?;
    unbond_batches_map().save(deps.storage, 0, &new_unbond(env.block.time.seconds()))?;
    FSM.set_initial_state(deps.storage, ContractState::Idle)?;
    LAST_IDLE_CALL.save(deps.storage, &0)?;
    LAST_ICA_CHANGE_HEIGHT.save(deps.storage, &0)?;
    TOTAL_LSM_SHARES.save(deps.storage, &0)?;
    BONDED_AMOUNT.save(deps.storage, &Uint128::zero())?;
    NON_NATIVE_REWARDS_CONFIG.save(deps.storage, &vec![])?;
    LAST_LSM_REDEEM.save(deps.storage, &0)?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    Ok(match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?)?,
        QueryMsg::Owner {} => to_json_binary(
            &cw_ownable::get_ownership(deps.storage)?
                .owner
                .unwrap_or(Addr::unchecked(""))
                .to_string(),
        )?,
        QueryMsg::PendingLSMShares {} => query_pending_lsm_shares(deps)?,
        QueryMsg::LSMSharesToRedeem {} => query_lsm_shares_to_redeem(deps)?,
        QueryMsg::TotalBonded {} => to_json_binary(&BONDED_AMOUNT.load(deps.storage)?)?,
        QueryMsg::ExchangeRate {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&query_exchange_rate(deps, &config)?)?
        }
        QueryMsg::CurrentUnbondBatch {} => query_current_unbond_batch(deps)?,
        QueryMsg::UnbondBatch { batch_id } => query_unbond_batch(deps, batch_id)?,
        QueryMsg::NonNativeRewardsReceivers {} => {
            to_json_binary(&NON_NATIVE_REWARDS_CONFIG.load(deps.storage)?)?
        }
        QueryMsg::ContractState {} => to_json_binary(&FSM.get_current_state(deps.storage)?)?,
        QueryMsg::LastPuppeteerResponse {} => to_json_binary(&LastPuppeteerResponse {
            response: LAST_PUPPETEER_RESPONSE.may_load(deps.storage)?,
        })?,
        QueryMsg::LastStakerResponse {} => to_json_binary(&LastStakerResponse {
            response: LAST_STAKER_RESPONSE.may_load(deps.storage)?,
        })?,
        QueryMsg::PauseInfo {} => query_pause_info(deps)?,
    })
}

fn query_pending_lsm_shares(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let shares: Vec<(String, (String, Uint128))> = PENDING_LSM_SHARES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    to_json_binary(&shares).map_err(From::from)
}

fn query_pause_info(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    if is_paused(deps.storage)? {
        to_json_binary(&PauseInfoResponse::Paused {}).map_err(From::from)
    } else {
        to_json_binary(&PauseInfoResponse::Unpaused {}).map_err(From::from)
    }
}

fn query_lsm_shares_to_redeem(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let shares: Vec<(String, (String, Uint128))> = LSM_SHARES_TO_REDEEM
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    to_json_binary(&shares).map_err(From::from)
}

fn query_exchange_rate(deps: Deps<NeutronQuery>, config: &Config) -> ContractResult<Decimal> {
    let fsm_state = FSM.get_current_state(deps.storage)?;
    if fsm_state != ContractState::Idle {
        return Ok(EXCHANGE_RATE
            .load(deps.storage)
            .unwrap_or((Decimal::one(), 0))
            .0);
    }
    let ld_total_supply: cosmwasm_std::SupplyResponse =
        deps.querier.query(&QueryRequest::Bank(BankQuery::Supply {
            denom: LD_DENOM.load(deps.storage)?,
        }))?;

    let exchange_rate_denominator = ld_total_supply.amount.amount;
    if exchange_rate_denominator.is_zero() {
        return Ok(Decimal::one());
    }

    let delegations = deps
        .querier
        .query_wasm_smart::<drop_staking_base::msg::puppeteer::DelegationsResponse>(
            &config.puppeteer_contract,
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

    let delegations_amount: Uint128 = delegations
        .0
        .delegations
        .iter()
        .map(|d| d.amount.amount)
        .sum();
    let mut batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    let mut unprocessed_unbonded_amount = Uint128::zero();
    let batch = unbond_batches_map().load(deps.storage, batch_id)?;
    if batch.status == UnbondBatchStatus::New {
        unprocessed_unbonded_amount += batch.total_amount;
    }
    if batch_id > 0 {
        batch_id -= 1;
        let batch = unbond_batches_map().load(deps.storage, batch_id)?;
        if batch.status == UnbondBatchStatus::UnbondRequested {
            unprocessed_unbonded_amount += batch.total_amount;
        }
    }
    let failed_batch_id = FAILED_BATCH_ID.may_load(deps.storage)?;
    if let Some(failed_batch_id) = failed_batch_id {
        let failed_batch = unbond_batches_map().load(deps.storage, failed_batch_id)?;
        unprocessed_unbonded_amount += failed_batch.total_amount;
    }
    let staker_balance: Uint128 = deps.querier.query_wasm_smart(
        &config.staker_contract,
        &drop_staking_base::msg::staker::QueryMsg::AllBalance {},
    )?;
    let total_lsm_shares = Uint128::new(TOTAL_LSM_SHARES.load(deps.storage)?);
    // arithmetic operations order is important here as we don't want to overflow
    let exchange_rate_numerator =
        delegations_amount + staker_balance + total_lsm_shares - unprocessed_unbonded_amount;
    if exchange_rate_numerator.is_zero() {
        return Ok(Decimal::one());
    }

    let exchange_rate = Decimal::from_ratio(exchange_rate_numerator, exchange_rate_denominator);
    Ok(exchange_rate)
}

fn cache_exchange_rate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    config: &Config,
) -> ContractResult<()> {
    let exchange_rate = query_exchange_rate(deps.as_ref(), config)?;
    EXCHANGE_RATE.save(deps.storage, &(exchange_rate, env.block.height))?;
    Ok(())
}

fn query_current_unbond_batch(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    to_json_binary(&UNBOND_BATCH_ID.load(deps.storage)?)
}

fn query_unbond_batch(deps: Deps<NeutronQuery>, batch_id: Uint128) -> StdResult<Binary> {
    to_json_binary(&unbond_batches_map().load(deps.storage, batch_id.u128())?)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond { receiver, r#ref } => execute_bond(deps, info, receiver, r#ref),
        ExecuteMsg::Unbond {} => execute_unbond(deps, info),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, *new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::ResetBondedAmount {} => execute_reset_bonded_amount(deps, env, info),
        ExecuteMsg::ProcessEmergencyBatch {
            batch_id,
            unbonded_amount,
        } => execute_process_emergency_batch(deps, info, env, batch_id, unbonded_amount),
        ExecuteMsg::UpdateNonNativeRewardsReceivers { items } => {
            execute_set_non_native_rewards_receivers(deps, env, info, items)
        }
        ExecuteMsg::Tick {} => execute_tick(deps, env, info),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
        ExecuteMsg::StakerHook(msg) => execute_staker_hook(deps, env, info, *msg),
        ExecuteMsg::Pause {} => exec_pause(deps, info),
        ExecuteMsg::Unpause {} => exec_unpause(deps, info),
    }
}

fn exec_pause(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    set_pause(deps.storage)?;

    Ok(response(
        "exec_pause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn exec_unpause(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    unpause(deps.storage);

    Ok(response(
        "exec_unpause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn execute_reset_bonded_amount(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    BONDED_AMOUNT.save(deps.storage, &Uint128::zero())?;
    Ok(response(
        "execute-reset_bond_limit",
        CONTRACT_NAME,
        vec![attr("action", "reset_bond_limit")],
    ))
}

fn execute_process_emergency_batch(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    env: Env,
    batch_id: u128,
    unbonded_amount: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    ensure_ne!(
        unbonded_amount,
        Uint128::zero(),
        ContractError::UnbondedAmountZero {}
    );

    let mut batch = unbond_batches_map().load(deps.storage, batch_id)?;
    ensure_eq!(
        batch.status,
        UnbondBatchStatus::WithdrawnEmergency,
        ContractError::BatchNotWithdrawnEmergency {}
    );
    ensure!(
        batch.expected_amount >= unbonded_amount,
        ContractError::UnbondedAmountTooHigh {}
    );

    let slashing_effect = Decimal::from_ratio(unbonded_amount, batch.expected_amount);
    batch.status = UnbondBatchStatus::Withdrawn;
    batch.unbonded_amount = Some(unbonded_amount);
    batch.slashing_effect = Some(slashing_effect);
    batch.status_timestamps.withdrawn = Some(env.block.time.seconds());
    unbond_batches_map().save(deps.storage, batch_id, &batch)?;

    Ok(response(
        "execute-process_emergency_batch",
        CONTRACT_NAME,
        vec![
            attr("action", "process_emergency_batch"),
            attr("batch_id", batch_id.to_string()),
            attr("unbonded_amount", unbonded_amount),
            attr("slashing_effect", slashing_effect.to_string()),
        ],
    ))
}

fn execute_set_non_native_rewards_receivers(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    items: Vec<NonNativeRewardsItem>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    for item in &items {
        if item.fee < Decimal::zero() || item.fee > Decimal::one() {
            return Err(ContractError::InvalidFee {});
        }
    }
    NON_NATIVE_REWARDS_CONFIG.save(deps.storage, &items)?;
    Ok(response(
        "execute-set_non_native_rewards_receivers",
        CONTRACT_NAME,
        vec![attr("action", "set_non_native_rewards_receivers")],
    ))
}

fn execute_staker_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_staking_base::msg::staker::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.staker_contract,
        ContractError::Unauthorized {}
    );
    LAST_STAKER_RESPONSE.save(deps.storage, &msg)?;
    Ok(response(
        "execute-staker_hook",
        CONTRACT_NAME,
        vec![attr("action", "staker_hook")],
    ))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );
    match msg.clone() {
        drop_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) => {
            LAST_ICA_CHANGE_HEIGHT.save(deps.storage, &success_msg.local_height)?;
            match &success_msg.transaction {
                drop_puppeteer_base::msg::Transaction::IBCTransfer {
                    denom,
                    amount,
                    reason,
                    recipient: _,
                } => {
                    if *reason == IBCTransferReason::LSMShare {
                        let current_pending =
                            PENDING_LSM_SHARES.may_load(deps.storage, denom.to_string())?;
                        if let Some((remote_denom, current_amount)) = current_pending {
                            let sent_amount = Uint128::from(*amount);
                            LSM_SHARES_TO_REDEEM.update(
                                deps.storage,
                                denom.to_string(),
                                |one| {
                                    let mut new = one.unwrap_or((remote_denom, Uint128::zero()));
                                    new.1 += sent_amount;
                                    StdResult::Ok(new)
                                },
                            )?;
                            if current_amount == sent_amount {
                                PENDING_LSM_SHARES.remove(deps.storage, denom.to_string());
                            } else {
                                PENDING_LSM_SHARES.update(
                                    deps.storage,
                                    denom.to_string(),
                                    |one| match one {
                                        Some(one) => {
                                            let mut new = one;
                                            new.1 -= Uint128::from(*amount);
                                            StdResult::Ok(new)
                                        }
                                        None => unreachable!("denom should be in the map"),
                                    },
                                )?;
                            }
                        }
                    }
                }
                drop_puppeteer_base::msg::Transaction::RedeemShares { items, .. } => {
                    let mut sum = 0u128;
                    for item in items {
                        sum += item.amount.u128();
                        LSM_SHARES_TO_REDEEM.remove(deps.storage, item.local_denom.to_string());
                    }
                    TOTAL_LSM_SHARES.update(deps.storage, |one| StdResult::Ok(one - sum))?;
                    LAST_LSM_REDEEM.save(deps.storage, &env.block.time.seconds())?;
                }
                _ => {}
            }
        }
        drop_puppeteer_base::msg::ResponseHookMsg::Error(err_msg) => {
            match err_msg.transaction {
                drop_puppeteer_base::msg::Transaction::Transfer { .. } // this one is for transfering non-native rewards
                | drop_puppeteer_base::msg::Transaction::RedeemShares { .. }
                | drop_puppeteer_base::msg::Transaction::ClaimRewardsAndOptionalyTransfer { .. } => { // this goes to idle and then ruled in tick_idle
                // IBC transfer for LSM shares and pending stake
                FSM.go_to(deps.storage, ContractState::Idle)?
            }
            drop_puppeteer_base::msg::Transaction::IBCTransfer { reason, .. } => {
                if reason == IBCTransferReason::LSMShare {
                    FSM.go_to(deps.storage, ContractState::Idle)?;
                }
            }
                _ => {}
            }
        }
    }

    LAST_PUPPETEER_RESPONSE.save(deps.storage, &msg)?;

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    ))
}

fn execute_tick(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    pause_guard(deps.storage)?;

    let current_state = FSM.get_current_state(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    check_latest_icq_responses(deps.as_ref(), config.puppeteer_contract.to_string())?;

    match current_state {
        ContractState::Idle => execute_tick_idle(deps.branch(), env, info, &config),
        //
        ContractState::LSMRedeem => execute_tick_peripheral(deps.branch(), env, info, &config),
        ContractState::LSMTransfer => execute_tick_peripheral(deps.branch(), env, info, &config),
        ContractState::NonNativeRewardsTransfer => {
            execute_tick_peripheral(deps.branch(), env, info, &config)
        }
        //
        ContractState::Claiming => execute_tick_claiming(deps.branch(), env, info, &config),
        ContractState::StakingBond => execute_tick_staking_bond(deps.branch(), env, info, &config),
        ContractState::Unbonding => execute_tick_unbonding(deps.branch(), env, info, &config),
        ContractState::StakingRewards => {
            execute_tick_staking_rewards(deps.branch(), env, info, &config)
        }
    }
}

fn execute_tick_idle(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "tick_idle")];
    let last_idle_call = LAST_IDLE_CALL.load(deps.storage)?;
    let mut messages = vec![];
    cache_exchange_rate(deps.branch(), env.clone(), config)?;
    if env.block.time.seconds() - last_idle_call < config.idle_min_interval {
        //process non-native rewards
        if let Some(transfer_msg) =
            get_non_native_rewards_and_fee_transfer_msg(deps.as_ref(), info.clone(), &env)?
        {
            messages.push(transfer_msg);
            FSM.go_to(deps.storage, ContractState::NonNativeRewardsTransfer)?;
        } else if let Some(lsm_msg) =
            get_pending_redeem_msg(deps.as_ref(), config, &env, info.funds.clone())?
        {
            messages.push(lsm_msg);
            FSM.go_to(deps.storage, ContractState::LSMRedeem)?;
        } else if let Some(lsm_msg) =
            get_pending_lsm_share_msg(deps.branch(), config, &env, info.funds.clone())?
        {
            messages.push(lsm_msg);
            FSM.go_to(deps.storage, ContractState::LSMTransfer)?;
        } else {
            //return error if none
            return Err(ContractError::IdleMinIntervalIsNotReached {});
        }
    } else {
        let unbonding_batches = unbond_batches_map()
            .idx
            .status
            .prefix(UnbondBatchStatus::Unbonding as u8)
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        ensure!(
            !is_unbonding_time_close(
                &unbonding_batches,
                env.block.time.seconds(),
                config.unbonding_safe_period
            ),
            ContractError::UnbondingTimeIsClose {}
        );

        let pump_ica_address = config
            .pump_ica_address
            .clone()
            .ok_or(ContractError::PumpIcaAddressIsNotSet {})?;
        let (ica_balance, _local_height, ica_balance_local_time) = get_ica_balance_by_denom(
            deps.as_ref(),
            config.puppeteer_contract.as_ref(),
            &config.remote_denom,
            true,
        )?;

        let unbonded_batches = if !unbonding_batches.is_empty() {
            unbonding_batches
                .into_iter()
                .filter(|(_id, batch)| {
                    batch.expected_release <= env.block.time.seconds()
                        && batch.expected_release < ica_balance_local_time
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let transfer: Option<TransferReadyBatchesMsg> = match unbonded_batches.len() {
            0 => None, // we have nothing to do
            1 => {
                let (id, mut unbonding_batch) = unbonded_batches
                    .into_iter()
                    // `.next().unwrap()` is safe to call since in this match arm
                    // `unbonding_batches` always has only 1 item
                    .next()
                    .unwrap();

                let (unbonded_amount, slashing_effect) =
                    if ica_balance < unbonding_batch.expected_amount {
                        (
                            ica_balance,
                            Decimal::from_ratio(ica_balance, unbonding_batch.expected_amount),
                        )
                    } else {
                        (unbonding_batch.expected_amount, Decimal::one())
                    };
                unbonding_batch.unbonded_amount = Some(unbonded_amount);
                unbonding_batch.slashing_effect = Some(slashing_effect);
                unbonding_batch.status = UnbondBatchStatus::Withdrawing;
                unbonding_batch.status_timestamps.withdrawing = Some(env.block.time.seconds());
                unbond_batches_map().save(deps.storage, id, &unbonding_batch)?;
                Some(TransferReadyBatchesMsg {
                    batch_ids: vec![id],
                    emergency: false,
                    amount: unbonded_amount,
                    recipient: pump_ica_address,
                })
            }
            _ => {
                let total_expected_amount: Uint128 = unbonded_batches
                    .iter()
                    .map(|(_id, batch)| batch.expected_amount)
                    .sum();
                let (emergency, recipient, amount) = if ica_balance < total_expected_amount {
                    (
                        true,
                        config
                            .emergency_address
                            .clone()
                            .ok_or(ContractError::EmergencyAddressIsNotSet {})?,
                        ica_balance,
                    )
                } else {
                    (false, pump_ica_address, total_expected_amount)
                };
                let mut batch_ids = vec![];
                for (id, mut batch) in unbonded_batches {
                    batch_ids.push(id);
                    if emergency {
                        batch.unbonded_amount = None;
                        batch.slashing_effect = None;
                        batch.status = UnbondBatchStatus::WithdrawingEmergency;
                        batch.status_timestamps.withdrawing_emergency =
                            Some(env.block.time.seconds());
                    } else {
                        batch.unbonded_amount = Some(batch.expected_amount);
                        batch.slashing_effect = Some(Decimal::one());
                        batch.status = UnbondBatchStatus::Withdrawing;
                        batch.status_timestamps.withdrawing = Some(env.block.time.seconds());
                    }
                    unbond_batches_map().save(deps.storage, id, &batch)?;
                }
                Some(TransferReadyBatchesMsg {
                    batch_ids,
                    emergency,
                    amount,
                    recipient,
                })
            }
        };

        let validators: Vec<ValidatorInfo> = deps.querier.query_wasm_smart(
            config.validators_set_contract.to_string(),
            &drop_staking_base::msg::validatorset::QueryMsg::Validators {},
        )?;

        let (delegations, local_height, _) =
            deps.querier
                .query_wasm_smart::<drop_staking_base::msg::puppeteer::DelegationsResponse>(
                    config.puppeteer_contract.to_string(),
                    &drop_puppeteer_base::msg::QueryMsg::Extension {
                        msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
                    },
                )?;

        ensure!(
            (env.block.height - local_height) <= config.icq_update_delay,
            ContractError::PuppeteerDelegationsOutdated {
                ica_height: env.block.height,
                control_height: local_height
            }
        );

        let validators_map = validators
            .iter()
            .map(|v| (v.valoper_address.clone(), v))
            .collect::<std::collections::HashMap<_, _>>();
        let validators_to_claim = delegations
            .delegations
            .iter()
            .filter(|d| validators_map.get(&d.validator).map_or(false, |_| true))
            .map(|d| d.validator.clone())
            .collect::<Vec<_>>();
        if validators_to_claim.is_empty() {
            attrs.push(attr("validators_to_claim", "empty"));
            if let Some(stake_bond_msg) = get_stake_bond_msg(deps.as_ref(), &env, config, &info)? {
                messages.push(stake_bond_msg);
                FSM.go_to(deps.storage, ContractState::StakingBond)?;
                attrs.push(attr("state", "staking_bond"));
            } else if let Some(stake_msg) =
                get_stake_rewards_msg(deps.as_ref(), &env, config, &info)?
            {
                messages.push(stake_msg);
                FSM.go_to(deps.storage, ContractState::StakingRewards)?;
                attrs.push(attr("state", "staking_rewards"));
            } else if let Some(unbond_message) =
                get_unbonding_msg(deps.branch(), &env, config, &info)?
            {
                messages.push(unbond_message);
                FSM.go_to(deps.storage, ContractState::Unbonding)?;
                attrs.push(attr("state", "unbonding"));
            } else {
                attrs.push(attr("state", "idle"));
            }
        } else {
            attrs.push(attr("validators_to_claim", validators_to_claim.join(",")));
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.puppeteer_contract.to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                        validators: validators_to_claim,
                        transfer,
                        reply_to: env.contract.address.to_string(),
                    },
                )?,
                funds: info.funds,
            }));
            FSM.go_to(deps.storage, ContractState::Claiming)?;
            attrs.push(attr("state", "claiming"));
        }
        LAST_IDLE_CALL.save(deps.storage, &env.block.time.seconds())?;
    }
    Ok(response("execute-tick_idle", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_peripheral(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    _config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    get_received_puppeteer_response(deps.as_ref())?;
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    let attrs = vec![attr("action", "tick_peripheral")];
    FSM.go_to(deps.storage, ContractState::Idle)?;
    Ok(response("execute-tick_peripheral", CONTRACT_NAME, attrs))
}

fn execute_tick_claiming(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let response_msg = get_received_puppeteer_response(deps.as_ref())?;
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    let mut attrs = vec![attr("action", "tick_claiming")];
    let mut messages = vec![];
    match response_msg {
        drop_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) => {
            match success_msg.transaction {
                drop_puppeteer_base::msg::Transaction::ClaimRewardsAndOptionalyTransfer {
                    transfer,
                    ..
                } => {
                    if let Some(transfer) = transfer {
                        for id in transfer.batch_ids {
                            let mut batch = unbond_batches_map().load(deps.storage, id)?;
                            attrs.push(attr("batch_id", id.to_string()));
                            if transfer.emergency {
                                batch.status = UnbondBatchStatus::WithdrawnEmergency;
                                batch.status_timestamps.withdrawing_emergency =
                                    Some(env.block.time.seconds());
                                attrs.push(attr("unbond_batch_status", "withdrawn_emergency"));
                            } else {
                                batch.status = UnbondBatchStatus::Withdrawn;
                                batch.status_timestamps.withdrawn = Some(env.block.time.seconds());
                                attrs.push(attr("unbond_batch_status", "withdrawn"));
                            }
                            unbond_batches_map().save(deps.storage, id, &batch)?;
                        }
                    }
                }
                _ => return Err(ContractError::InvalidTransaction {}),
            }
        }
        drop_puppeteer_base::msg::ResponseHookMsg::Error(err) => {
            attrs.push(attr("error_on_claiming", format!("{:?}", err)));
        }
    }
    if let Some(stake_bond_msg) = get_stake_bond_msg(deps.as_ref(), &env, config, &info)? {
        messages.push(stake_bond_msg);
        FSM.go_to(deps.storage, ContractState::StakingBond)?;
        attrs.push(attr("state", "staking_bond"));
    } else if let Some(stake_msg) = get_stake_rewards_msg(deps.as_ref(), &env, config, &info)? {
        messages.push(stake_msg);
        FSM.go_to(deps.storage, ContractState::StakingRewards)?;
        attrs.push(attr("state", "staking_rewards"));
    } else if let Some(unbond_message) = get_unbonding_msg(deps.branch(), &env, config, &info)? {
        messages.push(unbond_message);
        FSM.go_to(deps.storage, ContractState::Unbonding)?;
        attrs.push(attr("state", "unbonding"));
    } else {
        FSM.go_to(deps.storage, ContractState::Idle)?;
        attrs.push(attr("state", "idle"));
    }

    Ok(response("execute-tick_claiming", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_staking_bond(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let response_msg = get_received_staker_response(deps.as_ref())?;
    if let drop_staking_base::msg::staker::ResponseHookMsg::Success(response) = response_msg {
        let (_, puppeteer_height, _): drop_staking_base::msg::puppeteer::BalancesResponse =
            deps.querier.query_wasm_smart(
                config.puppeteer_contract.to_string(),
                &drop_puppeteer_base::msg::QueryMsg::Extension {
                    msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
                },
            )?;
        if response.local_height > puppeteer_height {
            return Err(ContractError::PuppeteerBalanceOutdated {
                ica_height: response.local_height,
                control_height: puppeteer_height,
            });
        }
    }
    LAST_STAKER_RESPONSE.remove(deps.storage);
    let mut messages = vec![];
    let mut attrs = vec![];
    attrs.push(attr("action", "tick_staking_bond"));
    if let Some(stake_msg) = get_stake_rewards_msg(deps.as_ref(), &env, config, &info)? {
        messages.push(stake_msg);
        FSM.go_to(deps.storage, ContractState::StakingRewards)?;
        attrs.push(attr("state", "staking_rewards"));
    } else if let Some(unbond_message) = get_unbonding_msg(deps.branch(), &env, config, &info)? {
        messages.push(unbond_message);
        FSM.go_to(deps.storage, ContractState::Unbonding)?;
        attrs.push(attr("state", "unbonding"));
    } else {
        FSM.go_to(deps.storage, ContractState::Idle)?;
        attrs.push(attr("state", "idle"));
    }
    Ok(response("execute-tick_transfering", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_staking_rewards(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let _response_msg = get_received_puppeteer_response(deps.as_ref())?;
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    let mut attrs = vec![attr("action", "tick_staking")];
    let mut messages = vec![];
    let unbond_message = get_unbonding_msg(deps.branch(), &env, config, &info)?;
    if let Some(unbond_message) = unbond_message {
        messages.push(unbond_message);
        FSM.go_to(deps.storage, ContractState::Unbonding)?;
        attrs.push(attr("state", "unbonding"));
    } else {
        FSM.go_to(deps.storage, ContractState::Idle)?;
        attrs.push(attr("state", "idle"));
    }
    Ok(response("execute-tick_staking", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_unbonding(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let res = get_received_puppeteer_response(deps.as_ref())?;
    let mut attrs = vec![attr("action", "tick_unbonding")];
    match res {
        drop_puppeteer_base::msg::ResponseHookMsg::Success(response) => {
            match response.transaction {
                drop_puppeteer_base::msg::Transaction::Undelegate { batch_id, .. } => {
                    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
                    attrs.push(attr("batch_id", batch_id.to_string()));
                    let mut unbond = unbond_batches_map().load(deps.storage, batch_id)?;
                    unbond.status = UnbondBatchStatus::Unbonding;
                    unbond.status_timestamps.unbonding = Some(env.block.time.seconds());
                    unbond.expected_release = env.block.time.seconds() + config.unbonding_period;
                    unbond_batches_map().save(deps.storage, batch_id, &unbond)?;
                    FAILED_BATCH_ID.remove(deps.storage);
                    attrs.push(attr("unbonding", "success"));
                }
                _ => return Err(ContractError::InvalidTransaction {}),
            }
        }
        drop_puppeteer_base::msg::ResponseHookMsg::Error(response) => match response.transaction {
            drop_puppeteer_base::msg::Transaction::Undelegate { batch_id, .. } => {
                LAST_PUPPETEER_RESPONSE.remove(deps.storage);
                attrs.push(attr("batch_id", batch_id.to_string()));
                let mut unbond = unbond_batches_map().load(deps.storage, batch_id)?;
                unbond.status = UnbondBatchStatus::UnbondFailed;
                unbond.status_timestamps.unbond_failed = Some(env.block.time.seconds());
                unbond_batches_map().save(deps.storage, batch_id, &unbond)?;
                FAILED_BATCH_ID.save(deps.storage, &batch_id)?;
                attrs.push(attr("unbonding", "failed"));
            }
            _ => return Err(ContractError::InvalidTransaction {}),
        },
    }
    FSM.go_to(deps.storage, ContractState::Idle)?;
    Ok(response("execute-tick_unbonding", CONTRACT_NAME, attrs))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    receiver: Option<String>,
    r#ref: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let Coin { amount, denom } = cw_utils::one_coin(&info)?;
    if let Some(bond_limit) = config.bond_limit {
        if BONDED_AMOUNT.load(deps.storage)? + amount > bond_limit {
            return Err(ContractError::BondLimitExceeded {});
        }
    }
    BONDED_AMOUNT.update(deps.storage, |total| StdResult::Ok(total + amount))?;
    let denom_type = check_denom::check_denom(&deps.as_ref(), &denom, &config)?;
    let mut msgs = vec![];
    let mut attrs = vec![attr("action", "bond")];
    let exchange_rate = query_exchange_rate(deps.as_ref(), &config)?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));

    if let check_denom::DenomType::LsmShare(remote_denom) = denom_type {
        if amount < config.lsm_min_bond_amount {
            return Err(ContractError::LSMBondAmountIsBelowMinimum {
                min_stake_amount: config.lsm_min_bond_amount,
                bond_amount: amount,
            });
        }
        TOTAL_LSM_SHARES.update(deps.storage, |total| StdResult::Ok(total + amount.u128()))?;
        PENDING_LSM_SHARES.update(deps.storage, denom, |one| {
            let mut new = one.unwrap_or((remote_denom, Uint128::zero()));
            new.1 += amount;
            StdResult::Ok(new)
        })?;
    } else {
        // if it's not LSM share, we send this amount to the staker
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.staker_contract.to_string(),
            amount: vec![Coin::new(amount.u128(), denom)],
        }));
    }

    let issue_amount = amount * (Decimal::one() / exchange_rate);
    attrs.push(attr("issue_amount", issue_amount.to_string()));

    let receiver = receiver.map_or(Ok::<String, ContractError>(info.sender.to_string()), |a| {
        deps.api.addr_validate(&a)?;
        Ok(a)
    })?;
    attrs.push(attr("receiver", receiver.clone()));
    if let Some(r#ref) = r#ref {
        if !r#ref.is_empty() {
            attrs.push(attr("ref", r#ref));
        }
    }
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.token_contract.into_string(),
        msg: to_json_binary(&TokenExecuteMsg::Mint {
            amount: issue_amount,
            receiver,
        })?,
        funds: vec![],
    }));
    Ok(response("execute-bond", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut attrs = vec![attr("action", "update_config")];
    if let Some(token_contract) = new_config.token_contract {
        config.token_contract = deps.api.addr_validate(&token_contract)?;
        attrs.push(attr("token_contract", token_contract));
    }
    if let Some(puppeteer_contract) = new_config.puppeteer_contract {
        config.puppeteer_contract = deps.api.addr_validate(&puppeteer_contract)?;
        attrs.push(attr("puppeteer_contract", puppeteer_contract));
    }
    if let Some(strategy_contract) = new_config.strategy_contract {
        config.strategy_contract = deps.api.addr_validate(&strategy_contract)?;
        attrs.push(attr("strategy_contract", strategy_contract));
    }
    if let Some(staker_contract) = new_config.staker_contract {
        config.staker_contract = deps.api.addr_validate(&staker_contract)?;
        attrs.push(attr("staker_contract", staker_contract));
    }
    if let Some(withdrawal_voucher_contract) = new_config.withdrawal_voucher_contract {
        config.withdrawal_voucher_contract =
            deps.api.addr_validate(&withdrawal_voucher_contract)?;
        attrs.push(attr(
            "withdrawal_voucher_contract",
            withdrawal_voucher_contract,
        ));
    }
    if let Some(withdrawal_manager_contract) = new_config.withdrawal_manager_contract {
        config.withdrawal_manager_contract =
            deps.api.addr_validate(&withdrawal_manager_contract)?;
        attrs.push(attr(
            "withdrawal_manager_contract",
            withdrawal_manager_contract,
        ));
    }
    if let Some(pump_ica_address) = new_config.pump_ica_address {
        attrs.push(attr("pump_address", &pump_ica_address));
        config.pump_ica_address = Some(pump_ica_address);
    }
    if let Some(transfer_channel_id) = new_config.transfer_channel_id {
        attrs.push(attr("transfer_channel_id", &transfer_channel_id));
        config.transfer_channel_id = transfer_channel_id;
    }
    if let Some(remote_denom) = new_config.remote_denom {
        attrs.push(attr("remote_denom", &remote_denom));
        config.remote_denom = remote_denom;
    }
    if let Some(validators_set_contract) = new_config.validators_set_contract {
        config.validators_set_contract = deps.api.addr_validate(&validators_set_contract)?;
        attrs.push(attr("validators_set_contract", validators_set_contract));
    }
    if let Some(base_denom) = new_config.base_denom {
        attrs.push(attr("base_denom", &base_denom));
        config.base_denom = base_denom;
    }
    if let Some(idle_min_interval) = new_config.idle_min_interval {
        attrs.push(attr("idle_min_interval", idle_min_interval.to_string()));
        config.idle_min_interval = idle_min_interval;
    }
    if let Some(unbonding_period) = new_config.unbonding_period {
        attrs.push(attr("unbonding_period", unbonding_period.to_string()));
        config.unbonding_period = unbonding_period;
    }
    if let Some(unbonding_safe_period) = new_config.unbonding_safe_period {
        attrs.push(attr(
            "unbonding_safe_period",
            unbonding_safe_period.to_string(),
        ));
        config.unbonding_safe_period = unbonding_safe_period;
    }
    if let Some(unbond_batch_switch_time) = new_config.unbond_batch_switch_time {
        attrs.push(attr(
            "unbond_batch_switch_time",
            unbond_batch_switch_time.to_string(),
        ));
        config.unbond_batch_switch_time = unbond_batch_switch_time;
    }
    if let Some(lsm_min_bond_amount) = new_config.lsm_min_bond_amount {
        attrs.push(attr("lsm_min_bond_amount", lsm_min_bond_amount.to_string()));
        config.lsm_min_bond_amount = lsm_min_bond_amount;
    }
    if let Some(lsm_redeem_maximum_interval) = new_config.lsm_redeem_maximum_interval {
        attrs.push(attr(
            "lsm_redeem_maximum_interval",
            lsm_redeem_maximum_interval.to_string(),
        ));
        config.lsm_redeem_maximum_interval = lsm_redeem_maximum_interval;
    }
    if let Some(lsm_redeem_threshold) = new_config.lsm_redeem_threshold {
        attrs.push(attr(
            "lsm_redeem_threshold",
            lsm_redeem_threshold.to_string(),
        ));
        config.lsm_redeem_threshold = lsm_redeem_threshold;
    }
    if let Some(bond_limit) = new_config.bond_limit {
        attrs.push(attr("bond_limit", bond_limit.to_string()));
        config.bond_limit = {
            if bond_limit.is_zero() {
                None
            } else {
                Some(bond_limit)
            }
        };
    }
    if let Some(fee) = new_config.fee {
        if fee < Decimal::zero() || fee > Decimal::one() {
            return Err(ContractError::InvalidFee {});
        }
        attrs.push(attr("fee", fee.to_string()));
        config.fee = Some(fee);
    }
    if let Some(fee_address) = new_config.fee_address {
        attrs.push(attr("fee_address", &fee_address));
        config.fee_address = Some(fee_address);
    }
    if let Some(emergency_address) = new_config.emergency_address {
        attrs.push(attr("emergency_address", &emergency_address));
        config.emergency_address = Some(emergency_address);
    }
    if let Some(min_stake_amount) = new_config.min_stake_amount {
        attrs.push(attr("min_stake_amount", min_stake_amount));
        config.min_stake_amount = min_stake_amount;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(response("execute-update_config", CONTRACT_NAME, attrs))
}

fn execute_unbond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "unbond")];
    let unbond_batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let ld_denom = LD_DENOM.load(deps.storage)?;
    let amount = cw_utils::must_pay(&info, &ld_denom)?;
    BONDED_AMOUNT.update(deps.storage, |total| StdResult::Ok(total - amount))?;
    let mut unbond_batch = unbond_batches_map().load(deps.storage, unbond_batch_id)?;
    let exchange_rate = query_exchange_rate(deps.as_ref(), &config)?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));
    let expected_amount = amount * exchange_rate;
    unbond_batch.total_unbond_items += 1;
    unbond_batch.total_amount += amount;
    unbond_batch.expected_amount += expected_amount;

    attrs.push(attr("expected_amount", expected_amount.to_string()));
    unbond_batches_map().save(deps.storage, unbond_batch_id, &unbond_batch)?;
    let extension = Some(Metadata {
        description: Some("Withdrawal voucher".into()),
        name: "LDV voucher".to_string(),
        batch_id: unbond_batch_id.to_string(),
        amount,
        expected_amount,
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: "unbond_batch_id".to_string(),
                value: unbond_batch_id.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "received_amount".to_string(),
                value: amount.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "expected_amount".to_string(),
                value: expected_amount.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "exchange_rate".to_string(),
                value: exchange_rate.to_string(),
            },
        ]),
    });
    let msgs = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.withdrawal_voucher_contract.into_string(),
            msg: to_json_binary(&VoucherExecuteMsg::Mint {
                owner: info.sender.to_string(),
                token_id: unbond_batch_id.to_string()
                    + "_"
                    + info.sender.to_string().as_str()
                    + "_"
                    + &unbond_batch.total_unbond_items.to_string(),
                token_uri: None,
                extension,
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.token_contract.into_string(),
            msg: to_json_binary(&TokenExecuteMsg::Burn {})?,
            funds: vec![Coin {
                denom: ld_denom,
                amount,
            }],
        }),
    ];
    Ok(response("execute-unbond", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn check_latest_icq_responses(
    deps: Deps<NeutronQuery>,
    puppeteer_contract: String,
) -> ContractResult<Response<NeutronMsg>> {
    let last_ica_balance_change_height = LAST_ICA_CHANGE_HEIGHT.load(deps.storage)?;

    let (_, balance_height, _): drop_staking_base::msg::puppeteer::BalancesResponse =
        deps.querier.query_wasm_smart(
            puppeteer_contract.to_string(),
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )?;

    ensure!(
        last_ica_balance_change_height <= balance_height,
        ContractError::PuppeteerBalanceOutdated {
            ica_height: last_ica_balance_change_height,
            control_height: balance_height
        }
    );

    let (_, delegations_height, _): drop_staking_base::msg::puppeteer::DelegationsResponse =
        deps.querier.query_wasm_smart(
            puppeteer_contract,
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

    ensure!(
        last_ica_balance_change_height <= delegations_height,
        ContractError::PuppeteerDelegationsOutdated {
            ica_height: last_ica_balance_change_height,
            control_height: delegations_height
        }
    );

    Ok(Response::new())
}

pub fn get_stake_bond_msg<T>(
    deps: Deps<NeutronQuery>,
    _env: &Env,
    config: &Config,
    info: &MessageInfo,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let staker_pending_stake: Result<Uint128, _> = deps.querier.query_wasm_smart(
        config.staker_contract.to_string(),
        &drop_staking_base::msg::staker::QueryMsg::NonStakedBalance {},
    );
    if let Ok(staker_pending_stake) = staker_pending_stake {
        if staker_pending_stake.is_zero() {
            return Ok(None);
        }
        let to_delegate: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
            &config.strategy_contract,
            &drop_staking_base::msg::strategy::QueryMsg::CalcDeposit {
                deposit: staker_pending_stake,
            },
        )?;
        return Ok(Some(CosmosMsg::<T>::Wasm(WasmMsg::Execute {
            contract_addr: config.staker_contract.to_string(),
            msg: to_json_binary(&drop_staking_base::msg::staker::ExecuteMsg::Stake {
                items: to_delegate,
            })?,
            funds: info.funds.clone(),
        })));
    }
    Ok(None)
}

pub fn get_stake_rewards_msg<T>(
    deps: Deps<NeutronQuery>,
    env: &Env,
    config: &Config,
    info: &MessageInfo,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let funds = info.funds.clone();
    let (balance, _, _) = get_ica_balance_by_denom(
        deps,
        config.puppeteer_contract.as_ref(),
        &config.remote_denom,
        true,
    )?;

    if balance < config.min_stake_amount {
        return Ok(None);
    }

    let fee = config.fee.unwrap_or(Decimal::zero()) * balance;
    let deposit_amount = balance - fee;

    let to_delegate: Vec<(String, Uint128)> = deps.querier.query_wasm_smart(
        &config.strategy_contract,
        &drop_staking_base::msg::strategy::QueryMsg::CalcDeposit {
            deposit: deposit_amount,
        },
    )?;

    let staking_fee: Option<(String, Uint128)> =
        if fee > Uint128::zero() && config.fee_address.is_some() {
            Some((config.fee_address.clone().unwrap(), fee))
        } else {
            None
        };

    Ok(Some(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract.to_string(),
        msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
            items: to_delegate,
            fee: staking_fee,
            reply_to: env.contract.address.to_string(),
        })?,
        funds,
    })))
}

fn get_unbonding_msg<T>(
    deps: DepsMut<NeutronQuery>,
    env: &Env,
    config: &Config,
    info: &MessageInfo,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let funds = info.funds.clone();
    let batch_id = FAILED_BATCH_ID
        .may_load(deps.storage)?
        .unwrap_or(UNBOND_BATCH_ID.load(deps.storage)?);
    let mut unbond = unbond_batches_map().load(deps.storage, batch_id)?;
    if (unbond.status_timestamps.new + config.unbond_batch_switch_time < env.block.time.seconds())
        && unbond.total_unbond_items != 0
        && !unbond.total_amount.is_zero()
    {
        let calc_withdraw_query_result: Result<Vec<(String, Uint128)>, StdError> =
            deps.querier.query_wasm_smart(
                config.strategy_contract.to_string(),
                &drop_staking_base::msg::strategy::QueryMsg::CalcWithdraw {
                    withdraw: unbond.total_amount,
                },
            );

        if calc_withdraw_query_result.is_err() {
            return Ok(None);
        }

        let undelegations: Vec<(String, Uint128)> = calc_withdraw_query_result?;

        unbond.status = UnbondBatchStatus::UnbondRequested;
        unbond.status_timestamps.unbond_requested = Some(env.block.time.seconds());
        unbond_batches_map().save(deps.storage, batch_id, &unbond)?;
        UNBOND_BATCH_ID.save(deps.storage, &(batch_id + 1))?;
        unbond_batches_map().save(
            deps.storage,
            batch_id + 1,
            &new_unbond(env.block.time.seconds()),
        )?;
        Ok(Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.puppeteer_contract.to_string(),
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
                items: undelegations,
                batch_id,
                reply_to: env.contract.address.to_string(),
            })?,
            funds,
        })))
    } else {
        Ok(None)
    }
}

fn get_received_puppeteer_response(
    deps: Deps<NeutronQuery>,
) -> ContractResult<drop_puppeteer_base::msg::ResponseHookMsg> {
    LAST_PUPPETEER_RESPONSE
        .load(deps.storage)
        .map_err(|_| ContractError::PuppeteerResponseIsNotReceived {})
}

fn get_received_staker_response(
    deps: Deps<NeutronQuery>,
) -> ContractResult<drop_staking_base::msg::staker::ResponseHookMsg> {
    LAST_STAKER_RESPONSE
        .load(deps.storage)
        .map_err(|_| ContractError::PuppeteerResponseIsNotReceived {})
}

fn is_unbonding_time_close(
    unbonding_batches: &[(u128, UnbondBatch)],
    now: u64,
    safe_period: u64,
) -> bool {
    for (_id, unbond_batch) in unbonding_batches {
        let expected = unbond_batch.expected_release;
        if (now < expected) && (now > expected - safe_period) {
            return true;
        }
    }
    false
}

fn get_ica_balance_by_denom<T: CustomQuery>(
    deps: Deps<T>,
    puppeteer_contract: &str,
    remote_denom: &str,
    can_be_zero: bool,
) -> ContractResult<(Uint128, u64, u64)> {
    let (ica_balances, balance_height, local_time): drop_staking_base::msg::puppeteer::BalancesResponse =
        deps.querier.query_wasm_smart(
            puppeteer_contract.to_string(),
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )?;

    let last_ica_balance_change_height = LAST_ICA_CHANGE_HEIGHT.load(deps.storage)?;
    ensure!(
        last_ica_balance_change_height <= balance_height,
        ContractError::PuppeteerBalanceOutdated {
            ica_height: last_ica_balance_change_height,
            control_height: balance_height
        }
    );

    let balance = ica_balances.coins.iter().find_map(|c| {
        if c.denom == remote_denom {
            Some(c.amount)
        } else {
            None
        }
    });
    Ok((
        match can_be_zero {
            true => balance.unwrap_or(Uint128::zero()),
            false => balance.ok_or(ContractError::ICABalanceZero {})?,
        },
        balance_height,
        local_time.seconds(),
    ))
}

fn new_unbond(now: u64) -> UnbondBatch {
    UnbondBatch {
        total_amount: Uint128::zero(),
        expected_amount: Uint128::zero(),
        total_unbond_items: 0,
        status: UnbondBatchStatus::New,
        expected_release: 0,
        slashing_effect: None,
        unbonded_amount: None,
        withdrawed_amount: None,
        status_timestamps: UnbondBatchStatusTimestamps {
            new: now,
            unbond_requested: None,
            unbond_failed: None,
            unbonding: None,
            withdrawing: None,
            withdrawn: None,
            withdrawing_emergency: None,
            withdrawn_emergency: None,
        },
    }
}

pub fn get_non_native_rewards_and_fee_transfer_msg<T>(
    deps: Deps<NeutronQuery>,
    info: MessageInfo,
    env: &Env,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let config = CONFIG.load(deps.storage)?;
    let non_native_rewards_receivers = NON_NATIVE_REWARDS_CONFIG.load(deps.storage)?;
    if non_native_rewards_receivers.is_empty() {
        return Ok(None);
    }
    let mut items = vec![];
    let rewards: drop_staking_base::msg::puppeteer::BalancesResponse =
        deps.querier.query_wasm_smart(
            config.puppeteer_contract.to_string(),
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::NonNativeRewardsBalances {},
            },
        )?;

    let last_ica_balance_change_height = LAST_ICA_CHANGE_HEIGHT.load(deps.storage)?;
    ensure!(
        last_ica_balance_change_height <= rewards.1,
        ContractError::PuppeteerBalanceOutdated {
            ica_height: last_ica_balance_change_height,
            control_height: rewards.1
        }
    );

    let rewards_map = rewards
        .0
        .coins
        .iter()
        .map(|c| (c.denom.clone(), c.amount))
        .collect::<std::collections::HashMap<_, _>>();
    let default_amount = Uint128::zero();

    for item in non_native_rewards_receivers {
        let amount = rewards_map.get(&item.denom).unwrap_or(&default_amount);
        if amount > &item.min_amount {
            let fee = item.fee * *amount;
            let amount = *amount - fee;
            if !amount.is_zero() {
                items.push((
                    item.address,
                    cosmwasm_std::Coin {
                        denom: item.denom.clone(),
                        amount,
                    },
                ));
            }
            if !item.fee.is_zero() && !fee.is_zero() {
                items.push((
                    item.fee_address,
                    cosmwasm_std::Coin {
                        denom: item.denom,
                        amount: fee,
                    },
                ));
            }
        }
    }

    if items.is_empty() {
        return Ok(None);
    }

    Ok(Some(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract.into_string(),
        msg: to_json_binary(&drop_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
            items,
            reply_to: env.contract.address.to_string(),
        })?,
        funds: info.funds,
    })))
}

fn get_pending_redeem_msg<T>(
    deps: Deps<NeutronQuery>,
    config: &Config,
    env: &Env,
    funds: Vec<cosmwasm_std::Coin>,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let pending_lsm_shares_count = LSM_SHARES_TO_REDEEM
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .count();
    let last_lsm_redeem = LAST_LSM_REDEEM.load(deps.storage)?;
    let lsm_redeem_threshold = config.lsm_redeem_threshold as usize;
    if pending_lsm_shares_count == 0
        || ((pending_lsm_shares_count < lsm_redeem_threshold)
            || (last_lsm_redeem + config.lsm_redeem_maximum_interval > env.block.time.seconds()))
    {
        return Ok(None);
    }
    let shares_to_redeeem = LSM_SHARES_TO_REDEEM
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let items = shares_to_redeeem
        .iter()
        .map(|(local_denom, (denom, amount))| RedeemShareItem {
            amount: *amount,
            local_denom: local_denom.to_string(),
            remote_denom: denom.to_string(),
        })
        .collect();
    Ok(Some(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract.to_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
                items,
                reply_to: env.contract.address.to_string(),
            },
        )?,
        funds,
    })))
}

fn get_pending_lsm_share_msg<T, X: CustomQuery>(
    deps: DepsMut<X>,
    config: &Config,
    env: &Env,
    funds: Vec<cosmwasm_std::Coin>,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let lsm_share: Option<(String, (String, Uint128))> = PENDING_LSM_SHARES.first(deps.storage)?;
    match lsm_share {
        Some((local_denom, (_remote_denom, amount))) => {
            Ok(Some(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.puppeteer_contract.to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
                        reason: IBCTransferReason::LSMShare,
                        reply_to: env.contract.address.to_string(),
                    },
                )?,
                funds: {
                    let mut all_funds = vec![cosmwasm_std::Coin {
                        denom: local_denom,
                        amount,
                    }];
                    all_funds.extend(funds);
                    all_funds
                },
            })))
        }
        None => Ok(None),
    }
}

pub mod check_denom {
    use super::*;

    #[derive(PartialEq, Debug)]
    pub enum DenomType {
        Base,
        LsmShare(String),
    }

    // XXX: cosmos_sdk_proto defines these structures for me,
    // yet they don't derive serde::de::DeserializeOwned,
    // so I have to redefine them here manually >:(

    #[cw_serde]
    pub struct QueryDenomTraceResponse {
        pub denom_trace: DenomTrace,
    }

    #[cw_serde]
    pub struct DenomTrace {
        pub path: String,
        pub base_denom: String,
    }

    fn query_denom_trace(
        deps: &Deps<NeutronQuery>,
        denom: impl Into<String>,
    ) -> StdResult<QueryDenomTraceResponse> {
        let denom = denom.into();
        deps.querier
            .query(&QueryRequest::Stargate {
                path: "/ibc.applications.transfer.v1.Query/DenomTrace".to_string(),
                data: cosmos_sdk_proto::ibc::applications::transfer::v1::QueryDenomTraceRequest {
                    hash: denom.clone(),
                }
                    .encode_to_vec()
                    .into(),
            })
            .map_err(|e| {
                StdError::generic_err(format!(
                    "Query denom trace for denom {denom} failed: {e}, perhaps, this is not an IBC denom?"
                ))
            })
    }

    pub fn check_denom(
        deps: &Deps<NeutronQuery>,
        denom: &str,
        config: &Config,
    ) -> ContractResult<DenomType> {
        if denom == config.base_denom {
            return Ok(DenomType::Base);
        }

        let trace = query_denom_trace(deps, denom)?.denom_trace;
        let (port, channel) = trace
            .path
            .split_once('/')
            .ok_or(ContractError::InvalidDenom {})?;
        if port != "transfer" || channel != config.transfer_channel_id {
            return Err(ContractError::InvalidDenom {});
        }

        let (validator, unbonding_index) = trace
            .base_denom
            .split_once('/')
            .ok_or(ContractError::InvalidDenom {})?;
        unbonding_index
            .parse::<u64>()
            .map_err(|_| ContractError::InvalidDenom {})?;

        let validator_info = deps
            .querier
            .query_wasm_smart::<drop_staking_base::msg::validatorset::ValidatorResponse>(
                &config.validators_set_contract,
                &drop_staking_base::msg::validatorset::QueryMsg::Validator {
                    valoper: validator.to_string(),
                },
            )?
            .validator;
        if validator_info.is_none() {
            return Err(ContractError::InvalidDenom {});
        }

        Ok(DenomType::LsmShare(trace.base_denom))
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
