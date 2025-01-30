use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, ensure, ensure_eq, ensure_ne, to_json_binary, Addr, Attribute, BankQuery, Binary, Coin,
    CosmosMsg, CustomQuery, Decimal, Deps, DepsMut, Env, MessageInfo, Order, QueryRequest,
    Response, StdError, StdResult, Uint128, Uint64, WasmMsg,
};
use cw_storage_plus::Bound;
use drop_helpers::{answer::response, is_paused};
use drop_puppeteer_base::{msg::TransferReadyBatchesMsg, peripheral_hook::IBCTransferReason};
use drop_staking_base::{
    error::core::{ContractError, ContractResult},
    msg::{
        core::{
            BondCallback, BondHook, ExecuteMsg, FailedBatchResponse, InstantiateMsg,
            LastPuppeteerResponse, MigrateMsg, QueryMsg,
        },
        token::{
            ConfigResponse as TokenConfigResponse, ExecuteMsg as TokenExecuteMsg,
            QueryMsg as TokenQueryMsg,
        },
        withdrawal_voucher::ExecuteMsg as VoucherExecuteMsg,
    },
    state::{
        core::{
            unbond_batches_map, Config, ConfigOptional, ContractState, Pause, UnbondBatch,
            UnbondBatchStatus, UnbondBatchStatusTimestamps, UnbondBatchesResponse, BOND_HOOKS,
            BOND_PROVIDERS, CONFIG, EXCHANGE_RATE, FAILED_BATCH_ID, FSM, LAST_ICA_CHANGE_HEIGHT,
            LAST_IDLE_CALL, LAST_PUPPETEER_RESPONSE, LD_DENOM, MAX_BOND_PROVIDERS, PAUSE,
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
pub const UNBOND_BATCHES_PAGINATION_DEFAULT_LIMIT: Uint64 = Uint64::new(100u64);
#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let attrs: Vec<Attribute> = vec![
        attr("factory_contract", &msg.factory_contract),
        attr("base_denom", &msg.base_denom),
        attr("owner", &msg.owner),
    ];
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;
    let config = msg.into_config(deps.as_ref().into_empty())?;
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, token_contract);
    CONFIG.save(deps.storage, &config)?;
    LD_DENOM.save(
        deps.storage,
        &deps
            .querier
            .query_wasm_smart::<TokenConfigResponse>(
                &addrs.token_contract,
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
    BOND_HOOKS.save(deps.storage, &vec![])?;
    BOND_PROVIDERS.init(deps.storage)?;
    PAUSE.save(deps.storage, &Pause::default())?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    Ok(match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?)?,
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?,
        QueryMsg::TotalBonded {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&query_total_bonded(deps, &config)?)?
        }
        QueryMsg::ExchangeRate {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&query_exchange_rate(deps, &config)?)?
        }
        QueryMsg::CurrentUnbondBatch {} => query_current_unbond_batch(deps)?,
        QueryMsg::UnbondBatch { batch_id } => query_unbond_batch(deps, batch_id)?,
        QueryMsg::UnbondBatches { limit, page_key } => query_unbond_batches(deps, limit, page_key)?,
        QueryMsg::ContractState {} => to_json_binary(&FSM.get_current_state(deps.storage)?)?,
        QueryMsg::LastPuppeteerResponse {} => to_json_binary(&LastPuppeteerResponse {
            response: LAST_PUPPETEER_RESPONSE.may_load(deps.storage)?,
        })?,
        QueryMsg::TotalAsyncTokens {} => to_json_binary(&query_total_async_tokens(deps)?)?,
        QueryMsg::FailedBatch {} => to_json_binary(&FailedBatchResponse {
            response: FAILED_BATCH_ID.may_load(deps.storage)?,
        })?,
        QueryMsg::Pause {} => to_json_binary(&PAUSE.load(deps.storage)?)?,
        QueryMsg::BondHooks {} => to_json_binary(
            &BOND_HOOKS
                .load(deps.storage)?
                .into_iter()
                .map(|addr| addr.into_string())
                .collect::<Vec<String>>(),
        )?,
        QueryMsg::BondProviders {} => to_json_binary(&query_bond_providers(deps)?)?,
    })
}

fn query_total_bonded(deps: Deps<NeutronQuery>, config: &Config) -> ContractResult<Uint128> {
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, puppeteer_contract);
    let delegations_response = deps
        .querier
        .query_wasm_smart::<drop_staking_base::msg::puppeteer::DelegationsResponse>(
        &addrs.puppeteer_contract,
        &drop_puppeteer_base::msg::QueryMsg::Extension {
            msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
        },
    )?;

    Ok(delegations_response
        .delegations
        .delegations
        .iter()
        .map(|d| d.amount.amount)
        .sum())
}

fn query_total_async_tokens(deps: Deps<NeutronQuery>) -> ContractResult<Uint128> {
    let mut total_async_tokens = Uint128::zero();
    let bond_providers = BOND_PROVIDERS.get_all_providers(deps.storage)?;
    for provider in bond_providers {
        let async_tokens_amount: Uint128 = deps.querier.query_wasm_smart(
            provider.to_string(),
            &drop_staking_base::msg::bond_provider::QueryMsg::AsyncTokensAmount {},
        )?;

        total_async_tokens += async_tokens_amount;
    }

    Ok(total_async_tokens)
}

fn query_bond_providers(deps: Deps<NeutronQuery>) -> ContractResult<Vec<Addr>> {
    Ok(BOND_PROVIDERS.get_all_providers(deps.storage)?)
}

fn query_exchange_rate(deps: Deps<NeutronQuery>, config: &Config) -> ContractResult<Decimal> {
    let fsm_state = FSM.get_current_state(deps.storage)?;
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, puppeteer_contract);
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

    let mut exchange_rate_denominator = ld_total_supply.amount.amount;
    if exchange_rate_denominator.is_zero() {
        return Ok(Decimal::one());
    }

    let delegations_response = deps
        .querier
        .query_wasm_smart::<drop_staking_base::msg::puppeteer::DelegationsResponse>(
        &addrs.puppeteer_contract,
        &drop_puppeteer_base::msg::QueryMsg::Extension {
            msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
        },
    )?;

    let delegations_amount: Uint128 = delegations_response
        .delegations
        .delegations
        .iter()
        .map(|d| d.amount.amount)
        .sum();
    let mut batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    let mut unprocessed_dasset_to_unbond = Uint128::zero();
    let batch = unbond_batches_map().load(deps.storage, batch_id)?;
    if batch.status == UnbondBatchStatus::New {
        unprocessed_dasset_to_unbond += batch.total_dasset_amount_to_withdraw;
    }
    if batch_id > 0 {
        batch_id -= 1;
        let batch = unbond_batches_map().load(deps.storage, batch_id)?;
        if batch.status == UnbondBatchStatus::UnbondRequested {
            unprocessed_dasset_to_unbond += batch.total_dasset_amount_to_withdraw;
        }
    }
    let failed_batch_id = FAILED_BATCH_ID.may_load(deps.storage)?;
    if let Some(failed_batch_id) = failed_batch_id {
        let failed_batch = unbond_batches_map().load(deps.storage, failed_batch_id)?;
        unprocessed_dasset_to_unbond += failed_batch.total_dasset_amount_to_withdraw;
    }
    exchange_rate_denominator += unprocessed_dasset_to_unbond;

    let total_async_tokens_amount = query_total_async_tokens(deps)?;

    // arithmetic operations order is important here as we don't want to overflow
    let exchange_rate_numerator = delegations_amount + total_async_tokens_amount;
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

fn query_unbond_batches(
    deps: Deps<NeutronQuery>,
    limit: Option<Uint64>,
    page_key: Option<Uint128>,
) -> ContractResult<Binary> {
    let limit = limit.unwrap_or(UNBOND_BATCHES_PAGINATION_DEFAULT_LIMIT);

    let page_key = page_key.map(|key| key.u128()).map(Bound::inclusive);
    let mut iter = unbond_batches_map().range(deps.storage, page_key, None, Order::Ascending);

    let usize_limit = if limit <= Uint64::MAX {
        limit.u64() as usize
    } else {
        return Err(ContractError::QueryUnbondBatchesLimitExceeded {});
    };

    let mut unbond_batches = vec![];
    for i in (&mut iter).take(usize_limit) {
        let (_, unbond_batch) = i?;
        unbond_batches.push(unbond_batch);
    }

    let next_page_key = iter
        .next()
        .transpose()?
        .map(|(batch_id, _)| Uint128::from(batch_id));

    Ok(to_json_binary(&UnbondBatchesResponse {
        unbond_batches,
        next_page_key,
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
        ExecuteMsg::Bond { receiver, r#ref } => execute_bond(deps, info, env, receiver, r#ref),
        ExecuteMsg::Unbond {} => execute_unbond(deps, info, env),
        ExecuteMsg::Tick {} => execute_tick(deps, env, info),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, *new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::ProcessEmergencyBatch {
            batch_id,
            unbonded_amount,
        } => execute_process_emergency_batch(deps, info, env, batch_id, unbonded_amount),
        ExecuteMsg::UpdateWithdrawnAmount {
            batch_id,
            withdrawn_amount,
        } => execute_update_withdrawn_amount(deps, env, info, batch_id, withdrawn_amount),
        ExecuteMsg::PeripheralHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
        ExecuteMsg::SetPause(pause) => execute_set_pause(deps, info, pause),
        ExecuteMsg::SetBondHooks { hooks } => execute_set_bond_hooks(deps, info, hooks),
        ExecuteMsg::AddBondProvider {
            bond_provider_address,
        } => execute_add_bond_provider(deps, info, bond_provider_address),
        ExecuteMsg::RemoveBondProvider {
            bond_provider_address,
        } => execute_remove_bond_provider(deps, info, bond_provider_address),
    }
}

fn execute_set_bond_hooks(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    hooks: Vec<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let hooks_validated = hooks
        .iter()
        .map(|addr| deps.api.addr_validate(addr))
        .collect::<StdResult<Vec<Addr>>>()?;
    BOND_HOOKS.save(deps.storage, &hooks_validated)?;

    let attributes = hooks
        .into_iter()
        .map(|addr| attr("contract", addr))
        .collect::<Vec<Attribute>>();

    Ok(response(
        "execute-set-bond-hooks",
        CONTRACT_NAME,
        attributes,
    ))
}

fn execute_add_bond_provider(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    bond_provider_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    if BOND_PROVIDERS.get_all_providers(deps.storage)?.len() as u64 >= MAX_BOND_PROVIDERS {
        return Err(ContractError::MaxBondProvidersReached {});
    }

    let bond_provider_address = deps.api.addr_validate(&bond_provider_address)?;

    BOND_PROVIDERS.add(deps.storage, bond_provider_address.clone())?;

    Ok(response(
        "execute-add_bond_provider",
        CONTRACT_NAME,
        vec![attr("bond_provider_address", bond_provider_address)],
    ))
}

fn execute_remove_bond_provider(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    bond_provider_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let bond_provider_address = deps.api.addr_validate(&bond_provider_address)?;
    let bond_provider_can_be_removed_response: bool = deps
        .querier
        .query_wasm_smart(
            bond_provider_address.clone(),
            &drop_staking_base::msg::bond_provider::QueryMsg::CanBeRemoved {},
        )
        .unwrap();
    if !bond_provider_can_be_removed_response {
        return Err(ContractError::BondProviderBalanceNotEmpty {});
    }

    BOND_PROVIDERS.remove(deps.storage, bond_provider_address.clone())?;

    Ok(response(
        "execute-remove_bond_provider",
        CONTRACT_NAME,
        vec![attr("bond_provider_address", bond_provider_address)],
    ))
}

fn execute_set_pause(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    pause: Pause,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    pause.bond.validate()?;
    pause.unbond.validate()?;
    pause.tick.validate()?;

    PAUSE.save(deps.storage, &pause)?;

    let attrs = vec![
        ("bond", pause.bond.to_string()),
        ("unbond", pause.unbond.to_string()),
        ("tick", pause.tick.to_string()),
    ];

    Ok(response("execute-set-pause", CONTRACT_NAME, attrs))
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
        batch.expected_native_asset_amount >= unbonded_amount,
        ContractError::UnbondedAmountTooHigh {}
    );

    let slashing_effect = Decimal::from_ratio(unbonded_amount, batch.expected_native_asset_amount);
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

fn execute_update_withdrawn_amount(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    batch_id: u128,
    withdrawn_amount: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let addrs =
        drop_helpers::get_contracts!(deps, config.factory_contract, withdrawal_manager_contract);
    if info.sender != addrs.withdrawal_manager_contract {
        return Err(ContractError::Unauthorized {});
    }

    let mut batch = unbond_batches_map().load(deps.storage, batch_id)?;
    ensure_eq!(
        batch.status,
        UnbondBatchStatus::Withdrawn,
        ContractError::BatchNotWithdrawn {}
    );
    batch.withdrawn_amount = Some(batch.withdrawn_amount.unwrap_or_default() + withdrawn_amount);
    unbond_batches_map().save(deps.storage, batch_id, &batch)?;

    Ok(response(
        "execute-update_withdrawn_amount",
        CONTRACT_NAME,
        vec![attr("action", "update_withdrawn_amount")],
    ))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::peripheral_hook::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, puppeteer_contract);
    let allowed_senders: Vec<_> = vec![deps.api.addr_validate(&addrs.puppeteer_contract)?]
        .into_iter()
        .chain(BOND_PROVIDERS.get_all_providers(deps.as_ref().storage)?)
        .collect();

    ensure!(
        allowed_senders.contains(&info.sender),
        ContractError::Unauthorized {}
    );

    match msg.clone() {
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(success_msg) => {
            LAST_ICA_CHANGE_HEIGHT.save(deps.storage, &success_msg.remote_height)?;
        }
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Error(err_msg) => {
            match err_msg.transaction {
                drop_puppeteer_base::peripheral_hook::Transaction::Transfer { .. } // this one is for transfering non-native rewards
                | drop_puppeteer_base::peripheral_hook::Transaction::RedeemShares { .. }
                | drop_puppeteer_base::peripheral_hook::Transaction::ClaimRewardsAndOptionalyTransfer { .. } => { // this goes to idle and then ruled in tick_idle
                // IBC transfer for LSM shares and pending stake
                FSM.go_to(deps.storage, ContractState::Idle)?
            }
            drop_puppeteer_base::peripheral_hook::Transaction::IBCTransfer { reason, .. } => {
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
    if is_paused!(PAUSE, deps, env, tick) {
        return Err(drop_helpers::pause::PauseError::Paused {}.into());
    }

    let current_state = FSM.get_current_state(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, puppeteer_contract);

    check_latest_icq_responses(deps.as_ref(), addrs.puppeteer_contract)?;

    match current_state {
        ContractState::Idle => execute_tick_idle(deps.branch(), env, info, &config),
        //
        ContractState::Peripheral => execute_tick_peripheral(deps.branch(), env, info, &config),
        //
        ContractState::Claiming => execute_tick_claiming(deps.branch(), env, info, &config),
        ContractState::Unbonding => execute_tick_unbonding(deps.branch(), env, info, &config),
    }
}

fn execute_tick_idle(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "tick_idle"), attr("knot", "000")];
    let last_idle_call = LAST_IDLE_CALL.load(deps.storage)?;
    let mut messages = vec![];
    cache_exchange_rate(deps.branch(), env.clone(), config)?;
    let addrs = drop_helpers::get_contracts!(
        deps,
        config.factory_contract,
        puppeteer_contract,
        validators_set_contract
    );
    attrs.push(attr("knot", "002"));
    attrs.push(attr("knot", "003"));
    if env.block.time.seconds() - last_idle_call < config.idle_min_interval {
        let provider = BOND_PROVIDERS.next(deps.storage)?;

        let can_process_on_idle = deps.querier.query_wasm_smart::<bool>(
            provider.to_string(),
            &drop_staking_base::msg::bond_provider::QueryMsg::CanProcessOnIdle {},
        );

        if can_process_on_idle.unwrap_or(false) {
            attrs.push(attr("knot", "036")); // provider can process on idle
            attrs.push(attr("used_bond_provider", provider.to_string()));
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: provider.to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::bond_provider::ExecuteMsg::ProcessOnIdle {},
                )?,
                funds: info.funds.clone(),
            });

            messages.push(msg);

            FSM.go_to(deps.storage, ContractState::Peripheral)?;
        }
    } else {
        LAST_IDLE_CALL.save(deps.storage, &env.block.time.seconds())?;
        attrs.push(attr("knot", "004"));
        let unbonding_batches = unbond_batches_map()
            .idx
            .status
            .prefix(UnbondBatchStatus::Unbonding as u8)
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        attrs.push(attr("knot", "005"));
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
        let (ica_balance, _remote_height, ica_balance_local_time) = get_ica_balance_by_denom(
            deps.as_ref(),
            &addrs.puppeteer_contract,
            &config.remote_denom,
            true,
        )?;

        let unbonded_batches = if !unbonding_batches.is_empty() {
            unbonding_batches
                .into_iter()
                .filter(|(_id, batch)| {
                    batch.expected_release_time <= env.block.time.seconds()
                        && batch.expected_release_time < ica_balance_local_time
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        attrs.push(attr("knot", "007"));
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
                    if ica_balance < unbonding_batch.expected_native_asset_amount {
                        (
                            ica_balance,
                            Decimal::from_ratio(
                                ica_balance,
                                unbonding_batch.expected_native_asset_amount,
                            ),
                        )
                    } else {
                        (unbonding_batch.expected_native_asset_amount, Decimal::one())
                    };
                unbonding_batch.unbonded_amount = Some(unbonded_amount);
                unbonding_batch.slashing_effect = Some(slashing_effect);
                unbonding_batch.status = UnbondBatchStatus::Withdrawing;
                unbonding_batch.status_timestamps.withdrawing = Some(env.block.time.seconds());
                unbond_batches_map().save(deps.storage, id, &unbonding_batch)?;
                attrs.push(attr("knot", "008"));
                Some(TransferReadyBatchesMsg {
                    batch_ids: vec![id],
                    emergency: false,
                    amount: unbonded_amount,
                    recipient: pump_ica_address,
                })
            }
            _ => {
                let total_native_asset_expected_amount: Uint128 = unbonded_batches
                    .iter()
                    .map(|(_id, batch)| batch.expected_native_asset_amount)
                    .sum();
                let (emergency, recipient, amount) =
                    if ica_balance < total_native_asset_expected_amount {
                        (
                            true,
                            config
                                .emergency_address
                                .clone()
                                .ok_or(ContractError::EmergencyAddressIsNotSet {})?,
                            ica_balance,
                        )
                    } else {
                        (false, pump_ica_address, total_native_asset_expected_amount)
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
                        batch.unbonded_amount = Some(batch.expected_native_asset_amount);
                        batch.slashing_effect = Some(Decimal::one());
                        batch.status = UnbondBatchStatus::Withdrawing;
                        batch.status_timestamps.withdrawing = Some(env.block.time.seconds());
                    }
                    unbond_batches_map().save(deps.storage, id, &batch)?;
                }
                attrs.push(attr("knot", "048"));
                Some(TransferReadyBatchesMsg {
                    batch_ids,
                    emergency,
                    amount,
                    recipient,
                })
            }
        };

        let validators: Vec<ValidatorInfo> = deps.querier.query_wasm_smart(
            addrs.validators_set_contract.to_string(),
            &drop_staking_base::msg::validatorset::QueryMsg::Validators {},
        )?;

        let delegations_response = deps
            .querier
            .query_wasm_smart::<drop_staking_base::msg::puppeteer::DelegationsResponse>(
            addrs.puppeteer_contract.to_string(),
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

        attrs.push(attr("knot", "009"));
        ensure!(
            (env.block.height - delegations_response.local_height) <= config.icq_update_delay,
            ContractError::PuppeteerDelegationsOutdated {
                ica_height: env.block.height,
                control_height: delegations_response.local_height
            }
        );

        let validators_map = validators
            .iter()
            .map(|v| (v.valoper_address.clone(), v))
            .collect::<std::collections::HashMap<_, _>>();
        let validators_to_claim = delegations_response
            .delegations
            .delegations
            .iter()
            .filter(|d| validators_map.get(&d.validator).map_or(false, |_| true))
            .map(|d| d.validator.clone())
            .collect::<Vec<_>>();

        attrs.push(attr("knot", "010"));
        if !validators_to_claim.is_empty() {
            attrs.push(attr("validators_to_claim", validators_to_claim.join(",")));
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addrs.puppeteer_contract.to_string(),
                msg: to_json_binary(
                    &drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                        validators: validators_to_claim,
                        transfer,
                        reply_to: env.contract.address.to_string(),
                    },
                )?,
                funds: info.funds,
            }));
            attrs.push(attr("knot", "011"));
            FSM.go_to(deps.storage, ContractState::Claiming)?;
            attrs.push(attr("knot", "012"));
            attrs.push(attr("state", "claiming"));
        }
    }

    Ok(response("execute-tick_idle", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_peripheral(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "tick_peripheral")];
    let res = get_received_puppeteer_response(deps.as_ref())?;
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, puppeteer_contract);
    if let drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(msg) = res {
        match msg.transaction {
            drop_puppeteer_base::peripheral_hook::Transaction::RedeemShares { .. } => {
                attrs.push(attr("knot", "037"))
            }
            drop_puppeteer_base::peripheral_hook::Transaction::IBCTransfer { .. } => {
                attrs.push(attr("knot", "038"));
            }
            drop_puppeteer_base::peripheral_hook::Transaction::Stake { .. } => {
                attrs.push(attr("knot", "039"));
            }
            _ => {}
        }

        let balances_response: drop_staking_base::msg::puppeteer::BalancesResponse =
            deps.querier.query_wasm_smart(
                addrs.puppeteer_contract.to_string(),
                &drop_puppeteer_base::msg::QueryMsg::Extension {
                    msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
                },
            )?;
        if msg.remote_height > balances_response.remote_height {
            return Err(ContractError::PuppeteerBalanceOutdated {
                ica_height: msg.remote_height,
                control_height: balances_response.remote_height,
            });
        }
    }
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);

    FSM.go_to(deps.storage, ContractState::Idle)?;
    attrs.push(attr("knot", "000"));
    attrs.push(attr("state", "idle"));

    Ok(response("execute-tick_peripheral", CONTRACT_NAME, attrs))
}

fn execute_tick_claiming(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "tick_claiming")];
    attrs.push(attr("knot", "012"));
    let response_msg = get_received_puppeteer_response(deps.as_ref())?;
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    let mut messages = vec![];
    match response_msg {
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(success_msg) => {
            attrs.push(attr("knot", "047"));
            match success_msg.transaction {
                drop_puppeteer_base::peripheral_hook::Transaction::ClaimRewardsAndOptionalyTransfer {
                    transfer,
                    ..
                } => {
                    attrs.push(attr("knot", "013"));
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
                            attrs.push(attr("knot", "014"));
                            unbond_batches_map().save(deps.storage, id, &batch)?;
                        }
                    }
                }
                _ => return Err(ContractError::InvalidTransaction {}),
            }
        }
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Error(err) => {
            attrs.push(attr("error_on_claiming", format!("{:?}", err)));
            match err.transaction {
                drop_puppeteer_base::peripheral_hook::Transaction::ClaimRewardsAndOptionalyTransfer {
                    transfer,
                    ..
                } => {
                    FSM.go_to(deps.storage, ContractState::Idle)?;
                    attrs.push(attr("knot", "050"));
                    attrs.push(attr("knot", "000"));
                    // revert batch status if there was a transfer of unbonded batches
                    if let Some(transfer) = transfer {
                        for id in transfer.batch_ids {
                            let mut batch = unbond_batches_map().load(deps.storage, id)?;
                            batch.status = UnbondBatchStatus::Unbonding;
                            unbond_batches_map().save(deps.storage, id, &batch)?;
                        }
                    }
                    return Ok(response("execute-tick_claiming", CONTRACT_NAME, attrs));
                }
                _ => return Err(ContractError::InvalidTransaction {}),
            }
        }
    }
    attrs.push(attr("knot", "015"));
    if let Some(unbond_message) = get_unbonding_msg(deps.branch(), &env, config, &info, &mut attrs)?
    {
        messages.push(unbond_message);
        attrs.push(attr("knot", "028"));
        FSM.go_to(deps.storage, ContractState::Unbonding)?;
        attrs.push(attr("knot", "029"));
        attrs.push(attr("state", "unbonding"));
    } else {
        FSM.go_to(deps.storage, ContractState::Idle)?;
        attrs.push(attr("knot", "000"));
        attrs.push(attr("state", "idle"));
    }

    Ok(response("execute-tick_claiming", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_unbonding(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "tick_unbonding"), attr("knot", "029")];
    let res = get_received_puppeteer_response(deps.as_ref())?;
    match res {
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(response) => {
            match response.transaction {
                drop_puppeteer_base::peripheral_hook::Transaction::Undelegate {
                    batch_id, ..
                } => {
                    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
                    attrs.push(attr("batch_id", batch_id.to_string()));
                    let mut unbond = unbond_batches_map().load(deps.storage, batch_id)?;
                    unbond.status = UnbondBatchStatus::Unbonding;
                    unbond.status_timestamps.unbonding = Some(env.block.time.seconds());
                    unbond.expected_release_time =
                        env.block.time.seconds() + config.unbonding_period;
                    unbond_batches_map().save(deps.storage, batch_id, &unbond)?;
                    FAILED_BATCH_ID.remove(deps.storage);
                    attrs.push(attr("knot", "030"));
                    attrs.push(attr("unbonding", "success"));
                }
                _ => return Err(ContractError::InvalidTransaction {}),
            }
        }
        drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Error(response) => match response
            .transaction
        {
            drop_puppeteer_base::peripheral_hook::Transaction::Undelegate { batch_id, .. } => {
                LAST_PUPPETEER_RESPONSE.remove(deps.storage);
                attrs.push(attr("batch_id", batch_id.to_string()));
                let mut unbond = unbond_batches_map().load(deps.storage, batch_id)?;
                unbond.status = UnbondBatchStatus::UnbondFailed;
                unbond.status_timestamps.unbond_failed = Some(env.block.time.seconds());
                unbond_batches_map().save(deps.storage, batch_id, &unbond)?;
                FAILED_BATCH_ID.save(deps.storage, &batch_id)?;
                attrs.push(attr("unbonding", "failed"));
                attrs.push(attr("knot", "031"));
            }
            _ => return Err(ContractError::InvalidTransaction {}),
        },
    }
    FSM.go_to(deps.storage, ContractState::Idle)?;
    attrs.push(attr("knot", "000"));
    attrs.push(attr("state", "idle"));
    Ok(response("execute-tick_unbonding", CONTRACT_NAME, attrs))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    env: Env,
    receiver: Option<String>,
    r#ref: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    if is_paused!(PAUSE, deps, env, bond) {
        return Err(drop_helpers::pause::PauseError::Paused {}.into());
    }

    let config = CONFIG.load(deps.storage)?;
    let bonded_coin = cw_utils::one_coin(&info)?;
    let addrs = drop_helpers::get_contracts!(deps, config.factory_contract, token_contract);
    let Coin { amount, denom } = bonded_coin.clone();
    let mut msgs = vec![];
    let mut attrs = vec![attr("action", "bond")];
    let exchange_rate = query_exchange_rate(deps.as_ref(), &config)?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));

    let bond_providers = BOND_PROVIDERS.get_all_providers(deps.as_ref().storage)?;
    let mut bonded = false;
    for provider in bond_providers {
        let can_bond = deps.querier.query_wasm_smart::<bool>(
            provider.to_string(),
            &drop_staking_base::msg::bond_provider::QueryMsg::CanBond {
                denom: denom.clone(),
            },
        );
        if can_bond.unwrap_or(false) {
            attrs.push(attr("used_bond_provider", provider.to_string()));
            let issue_amount: Uint128 = deps.querier.query_wasm_smart(
                provider.to_string(),
                &drop_staking_base::msg::bond_provider::QueryMsg::TokensAmount {
                    coin: Coin::new(amount.u128(), denom.clone()),
                    exchange_rate,
                },
            )?;
            attrs.push(attr("issue_amount", issue_amount.to_string()));

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: provider.to_string(),
                msg: to_json_binary(&drop_staking_base::msg::bond_provider::ExecuteMsg::Bond {})?,
                funds: vec![Coin::new(amount.u128(), denom.clone())],
            });

            msgs.push(msg);

            let receiver = receiver.clone().map_or(
                Ok::<String, ContractError>(info.sender.to_string()),
                |a| {
                    deps.api.addr_validate(&a)?;
                    Ok(a)
                },
            )?;
            attrs.push(attr("receiver", receiver.clone()));
            if let Some(r#ref) = r#ref.clone() {
                if !r#ref.is_empty() {
                    attrs.push(attr("ref", r#ref));
                }
            }
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addrs.token_contract.to_string(),
                msg: to_json_binary(&TokenExecuteMsg::Mint {
                    amount: issue_amount,
                    receiver,
                })?,
                funds: vec![],
            }));

            let bond_hooks = BOND_HOOKS.load(deps.storage)?;
            if !bond_hooks.is_empty() {
                let hook_msg = BondHook {
                    amount: bonded_coin.amount,
                    denom: bonded_coin.denom,
                    sender: info.sender,
                    dasset_minted: issue_amount,
                    r#ref,
                };
                for hook in bond_hooks {
                    let msg = WasmMsg::Execute {
                        contract_addr: hook.into_string(),
                        msg: to_json_binary(&BondCallback::BondCallback(hook_msg.clone()))?,
                        funds: vec![],
                    };
                    msgs.push(msg.into());
                }
            }

            bonded = true;
            break;
        }
    }

    ensure!(
        bonded,
        ContractError::BondProviderError {
            message: "No sufficient bond provider found".into()
        }
    );

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
    if let Some(factory_contract) = new_config.factory_contract {
        config.factory_contract = deps.api.addr_validate(&factory_contract)?;
        attrs.push(attr("factory_contract", factory_contract));
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

    if let Some(emergency_address) = new_config.emergency_address {
        attrs.push(attr("emergency_address", &emergency_address));
        config.emergency_address = Some(emergency_address);
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(response("execute-update_config", CONTRACT_NAME, attrs))
}

fn execute_unbond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    env: Env,
) -> ContractResult<Response<NeutronMsg>> {
    if is_paused!(PAUSE, deps, env, unbond) {
        return Err(drop_helpers::pause::PauseError::Paused {}.into());
    }

    let attrs = vec![attr("action", "unbond")];
    let unbond_batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let ld_denom = LD_DENOM.load(deps.storage)?;
    let dasset_amount = cw_utils::must_pay(&info, &ld_denom)?;
    let addrs = drop_helpers::get_contracts!(
        deps,
        config.factory_contract,
        withdrawal_voucher_contract,
        token_contract
    );
    let mut unbond_batch = unbond_batches_map().load(deps.storage, unbond_batch_id)?;
    unbond_batch.total_unbond_items += 1;
    unbond_batch.total_dasset_amount_to_withdraw += dasset_amount;
    unbond_batches_map().save(deps.storage, unbond_batch_id, &unbond_batch)?;

    let extension = Some(Metadata {
        description: Some("Withdrawal voucher".into()),
        name: "LDV voucher".to_string(),
        batch_id: unbond_batch_id.to_string(),
        amount: dasset_amount,
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: "unbond_batch_id".to_string(),
                value: unbond_batch_id.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "received_amount".to_string(),
                value: dasset_amount.to_string(),
            },
        ]),
    });

    let msgs = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addrs.withdrawal_voucher_contract,
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
            contract_addr: addrs.token_contract,
            msg: to_json_binary(&TokenExecuteMsg::Burn {})?,
            funds: vec![Coin {
                denom: ld_denom,
                amount: dasset_amount,
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

    let balances_response: drop_staking_base::msg::puppeteer::BalancesResponse =
        deps.querier.query_wasm_smart(
            puppeteer_contract.to_string(),
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )?;

    ensure!(
        last_ica_balance_change_height <= balances_response.remote_height,
        ContractError::PuppeteerBalanceOutdated {
            ica_height: last_ica_balance_change_height,
            control_height: balances_response.remote_height
        }
    );

    let delegations_response: drop_staking_base::msg::puppeteer::DelegationsResponse =
        deps.querier.query_wasm_smart(
            puppeteer_contract,
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

    ensure!(
        last_ica_balance_change_height <= delegations_response.remote_height,
        ContractError::PuppeteerDelegationsOutdated {
            ica_height: last_ica_balance_change_height,
            control_height: delegations_response.remote_height
        }
    );

    Ok(Response::new())
}

fn get_unbonding_msg<T>(
    deps: DepsMut<NeutronQuery>,
    env: &Env,
    config: &Config,
    info: &MessageInfo,
    attrs: &mut Vec<cosmwasm_std::Attribute>,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let addrs = drop_helpers::get_contracts!(
        deps,
        config.factory_contract,
        strategy_contract,
        puppeteer_contract
    );
    let funds = info.funds.clone();
    attrs.push(attr("knot", "024"));
    let (batch_id, processing_failed_batch) = match FAILED_BATCH_ID.may_load(deps.storage)? {
        Some(batch_id) => (batch_id, true),
        None => (UNBOND_BATCH_ID.load(deps.storage)?, false),
    };
    let mut unbond = unbond_batches_map().load(deps.storage, batch_id)?;
    if processing_failed_batch {
        attrs.push(attr("knot", "025"));
    } else {
        attrs.push(attr("knot", "026"));
    }
    attrs.push(attr("knot", "027"));
    if (unbond.status_timestamps.new + config.unbond_batch_switch_time < env.block.time.seconds())
        && unbond.total_unbond_items != 0
        && !unbond.total_dasset_amount_to_withdraw.is_zero()
    {
        let current_exchange_rate = query_exchange_rate(deps.as_ref(), config)?;
        attrs.push(attr("exchange_rate", current_exchange_rate.to_string()));
        let expected_native_asset_amount =
            unbond.total_dasset_amount_to_withdraw * current_exchange_rate;

        let calc_withdraw_query_result: Result<Vec<(String, Uint128)>, StdError> =
            deps.querier.query_wasm_smart(
                addrs.strategy_contract,
                &drop_staking_base::msg::strategy::QueryMsg::CalcWithdraw {
                    withdraw: expected_native_asset_amount,
                },
            );

        if calc_withdraw_query_result.is_err() {
            return Ok(None);
        }

        let undelegations: Vec<(String, Uint128)> = calc_withdraw_query_result?;

        attrs.push(attr("knot", "045"));
        unbond.status = UnbondBatchStatus::UnbondRequested;
        unbond.status_timestamps.unbond_requested = Some(env.block.time.seconds());
        unbond.expected_native_asset_amount = expected_native_asset_amount;
        unbond_batches_map().save(deps.storage, batch_id, &unbond)?;

        attrs.push(attr("knot", "049"));
        if !processing_failed_batch {
            attrs.push(attr("knot", "046"));
            UNBOND_BATCH_ID.save(deps.storage, &(batch_id + 1))?;
            unbond_batches_map().save(
                deps.storage,
                batch_id + 1,
                &new_unbond(env.block.time.seconds()),
            )?;
        }
        Ok(Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addrs.puppeteer_contract,
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
) -> ContractResult<drop_puppeteer_base::peripheral_hook::ResponseHookMsg> {
    LAST_PUPPETEER_RESPONSE
        .load(deps.storage)
        .map_err(|_| ContractError::PuppeteerResponseIsNotReceived {})
}

fn is_unbonding_time_close(
    unbonding_batches: &[(u128, UnbondBatch)],
    now: u64,
    safe_period: u64,
) -> bool {
    for (_id, unbond_batch) in unbonding_batches {
        let expected = unbond_batch.expected_release_time;
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
    let balances_response: drop_staking_base::msg::puppeteer::BalancesResponse =
        deps.querier.query_wasm_smart(
            puppeteer_contract.to_string(),
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )?;

    let last_ica_balance_change_height = LAST_ICA_CHANGE_HEIGHT.load(deps.storage)?;
    ensure!(
        last_ica_balance_change_height <= balances_response.remote_height,
        ContractError::PuppeteerBalanceOutdated {
            ica_height: last_ica_balance_change_height,
            control_height: balances_response.remote_height
        }
    );

    let balance = balances_response.balances.coins.iter().find_map(|c| {
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
        balances_response.remote_height,
        balances_response.timestamp.seconds(),
    ))
}

fn new_unbond(now: u64) -> UnbondBatch {
    UnbondBatch {
        total_dasset_amount_to_withdraw: Uint128::zero(),
        expected_native_asset_amount: Uint128::zero(),
        total_unbond_items: 0,
        status: UnbondBatchStatus::New,
        expected_release_time: 0,
        slashing_effect: None,
        unbonded_amount: None,
        withdrawn_amount: None,
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

pub mod check_denom {
    use super::*;

    #[derive(PartialEq, Debug)]
    pub enum DenomType {
        Base,
        LsmShare(String, String),
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
        let addrs =
            drop_helpers::get_contracts!(deps, config.factory_contract, validators_set_contract);

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
                &addrs.validators_set_contract,
                &drop_staking_base::msg::validatorset::QueryMsg::Validator {
                    valoper: validator.to_string(),
                },
            )?
            .validator;
        if validator_info.is_none() {
            return Err(ContractError::InvalidDenom {});
        }

        Ok(DenomType::LsmShare(
            trace.base_denom.to_string(),
            validator.to_string(),
        ))
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
        deps.storage.remove("pause".as_bytes());
        PAUSE.save(deps.storage, &Pause::default())?;
        BOND_HOOKS.save(deps.storage, &vec![])?;
    }

    Ok(Response::new())
}
