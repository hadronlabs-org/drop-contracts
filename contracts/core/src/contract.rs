use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{
    attr, ensure, ensure_eq, ensure_ne, entry_point, to_json_binary, Attribute, BankQuery, Binary,
    CosmosMsg, CustomQuery, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use lido_helpers::answer::response;
use lido_puppeteer_base::msg::TransferReadyBatchMsg;
use lido_staking_base::state::core::{
    Config, ConfigOptional, ContractState, NonNativeRewardsItem, UnbondBatch, UnbondBatchStatus,
    UnbondItem, CONFIG, FAILED_BATCH_ID, FSM, LAST_ICA_BALANCE_CHANGE_HEIGHT,
    LAST_PUPPETEER_RESPONSE, NON_NATIVE_REWARDS_CONFIG, PENDING_TRANSFER, PRE_UNBONDING_BALANCE,
    UNBOND_BATCHES, UNBOND_BATCH_ID,
};
use lido_staking_base::state::validatorset::ValidatorInfo;
use lido_staking_base::state::withdrawal_voucher::{Metadata, Trait};
use lido_staking_base::{
    msg::{
        core::{ExecuteMsg, InstantiateMsg, QueryMsg},
        token::ExecuteMsg as TokenExecuteMsg,
        withdrawal_voucher::ExecuteMsg as VoucherExecuteMsg,
    },
    state::core::LAST_IDLE_CALL,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::str::FromStr;
use std::vec;
const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let attrs: Vec<Attribute> = vec![
        attr("token_contract", &msg.token_contract),
        attr("puppeteer_contract", &msg.puppeteer_contract),
        attr("strategy_contract", &msg.strategy_contract),
        attr("base_denom", &msg.base_denom),
        attr("owner", &msg.owner),
    ];
    CONFIG.save(deps.storage, &msg.into())?;
    //an empty unbonding batch added as it's ready to be used on unbond action
    UNBOND_BATCH_ID.save(deps.storage, &0)?;
    UNBOND_BATCHES.save(deps.storage, 0, &new_unbond(env.block.time.seconds()))?;
    FSM.set_initial_state(deps.storage, ContractState::Idle)?;
    LAST_IDLE_CALL.save(deps.storage, &0)?;
    LAST_ICA_BALANCE_CHANGE_HEIGHT.save(deps.storage, &0)?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    Ok(match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?)?,
        QueryMsg::ExchangeRate {} => to_json_binary(&query_exchange_rate(deps, env, None)?)?,
        QueryMsg::UnbondBatch { batch_id } => query_unbond_batch(deps, batch_id)?,
        QueryMsg::NonNativeRewardsReceivers {} => {
            to_json_binary(&NON_NATIVE_REWARDS_CONFIG.load(deps.storage)?)?
        }
        QueryMsg::ContractState {} => to_json_binary(&FSM.get_current_state(deps.storage)?)?,
        QueryMsg::LastPuppeteerResponse {} => {
            to_json_binary(&LAST_PUPPETEER_RESPONSE.load(deps.storage)?)?
        }
    })
}

fn query_exchange_rate(
    deps: Deps<NeutronQuery>,
    env: Env,
    current_stake: Option<Uint128>,
) -> ContractResult<Decimal> {
    let config = CONFIG.load(deps.storage)?;
    let ld_denom = config.ld_denom.ok_or(ContractError::LDDenomIsNotSet {})?;
    let ld_total_supply: cosmwasm_std::SupplyResponse =
        deps.querier
            .query(&cosmwasm_std::QueryRequest::Bank(BankQuery::Supply {
                denom: ld_denom,
            }))?;
    let ld_total_amount = ld_total_supply.amount.amount;
    if ld_total_amount.is_zero() {
        return Ok(Decimal::one());
    }
    let delegations = deps
        .querier
        .query_wasm_smart::<lido_staking_base::msg::puppeteer::DelegationsResponse>(
            config.puppeteer_contract.to_string(),
            &lido_puppeteer_base::msg::QueryMsg::Extention {
                msg: lido_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
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
    let batch = UNBOND_BATCHES.load(deps.storage, batch_id)?;
    if batch.status == UnbondBatchStatus::New {
        unprocessed_unbonded_amount += batch.total_amount;
    }
    if batch_id > 0 {
        batch_id -= 1;
        let batch = UNBOND_BATCHES.load(deps.storage, batch_id)?;
        if batch.status == UnbondBatchStatus::UnbondRequested {
            unprocessed_unbonded_amount += batch.total_amount;
        }
    }
    let failed_batch_id = FAILED_BATCH_ID.may_load(deps.storage)?;
    if let Some(failed_batch_id) = failed_batch_id {
        let failed_batch = UNBOND_BATCHES.load(deps.storage, failed_batch_id)?;
        unprocessed_unbonded_amount += failed_batch.total_amount;
    }
    let core_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), config.base_denom)?
        .amount;
    let extra_amount = match FSM.get_current_state(deps.storage)? {
        ContractState::Transfering => PENDING_TRANSFER.load(deps.storage),
        ContractState::Staking => {
            let (ica_balance, _) = get_ica_balance_by_denom(
                deps,
                &config.puppeteer_contract,
                &config.remote_denom,
                false,
            )?;
            Ok(ica_balance)
        }
        _ => Ok(Uint128::zero()),
    }?;
    Ok(Decimal::from_ratio(
        delegations_amount + core_balance + extra_amount
            - current_stake.unwrap_or(Uint128::zero())
            - unprocessed_unbonded_amount,
        ld_total_amount,
    )) // arithmetic operations order is important here as we don't want to overflow
}

fn query_unbond_batch(deps: Deps<NeutronQuery>, batch_id: Uint128) -> StdResult<Binary> {
    to_json_binary(&UNBOND_BATCHES.load(deps.storage, batch_id.into())?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond { receiver } => execute_bond(deps, env, info, receiver),
        ExecuteMsg::Unbond {} => execute_unbond(deps, env, info),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, *new_config),
        ExecuteMsg::UpdateNonNativeRewardsReceivers { items } => {
            execute_set_non_native_rewards_receivers(deps, env, info, items)
        }
        ExecuteMsg::FakeProcessBatch {
            batch_id,
            unbonded_amount,
        } => execute_fake_process_batch(deps, env, info, batch_id, unbonded_amount),
        ExecuteMsg::Tick {} => execute_tick(deps, env, info),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
    }
}

fn execute_set_non_native_rewards_receivers(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    items: Vec<NonNativeRewardsItem>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, config.owner, ContractError::Unauthorized {});
    NON_NATIVE_REWARDS_CONFIG.save(deps.storage, &items)?;
    Ok(response(
        "execute-set_non_native_rewards_receivers",
        CONTRACT_NAME,
        vec![attr("action", "set_non_native_rewards_receivers")],
    ))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: lido_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );
    deps.api
        .debug(&format!("WASMDEBUG puppeteer_hook {:?}", msg));
    if let lido_puppeteer_base::msg::ResponseHookMsg::Success(_) = msg {
        LAST_ICA_BALANCE_CHANGE_HEIGHT.save(deps.storage, &env.block.height)?;
    } // if it's error we don't need to save the height because balance wasn't changed
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
    let current_state = FSM.get_current_state(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    deps.api
        .debug(&format!("WASMDEBUG tick {:?}", current_state));
    match current_state {
        ContractState::Idle => execute_tick_idle(deps.branch(), env, info, &config),
        ContractState::Claiming => execute_tick_claiming(deps.branch(), env, info, &config),
        ContractState::Transfering => execute_tick_transfering(deps.branch(), env, info, &config),
        ContractState::Unbonding => execute_tick_unbonding(deps.branch(), env, info, &config),
        ContractState::Staking => execute_tick_staking(deps.branch(), env, info, &config),
    }
}

fn execute_tick_idle(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "tick_idle")];
    let last_idle_call = LAST_IDLE_CALL.load(deps.storage)?;
    let idle_min_interval = config.idle_min_interval;
    let mut messages = vec![];
    if env.block.time.seconds() - last_idle_call < idle_min_interval {
        //process non-native rewards
        if let Some(transfer_msg) = get_non_native_rewards_transfer_msg(deps.as_ref(), info, env)? {
            messages.push(transfer_msg);
        }
        //return error if none
        return Err(ContractError::IdleMinIntervalIsNotReached {});
    } else {
        ensure!(
            !is_unbonding_time_close(
                deps.as_ref(),
                &env.block.time.seconds(),
                &config.unbonding_safe_period
            )?,
            ContractError::UnbondingTimeIsClose {}
        );
        let pump_address = config
            .pump_address
            .clone()
            .ok_or(ContractError::PumpAddressIsNotSet {})?;
        // process unbond if any aleready unbonded
        // and claim rewards
        let transfer: Option<TransferReadyBatchMsg> = match get_unbonded_batch(deps.as_ref())? {
            Some((batch_id, batch)) => Some(TransferReadyBatchMsg {
                batch_id,
                amount: batch
                    .unbonded_amount
                    .ok_or(ContractError::UnbondedAmountIsNotSet {})?,
                recipient: pump_address,
            }),
            None => None,
        };

        let validators: Vec<ValidatorInfo> = deps.querier.query_wasm_smart(
            config.validators_set_contract.to_string(),
            &lido_staking_base::msg::validatorset::QueryMsg::Validators {},
        )?;

        let (delegations, _) = deps
            .querier
            .query_wasm_smart::<lido_staking_base::msg::puppeteer::DelegationsResponse>(
            config.puppeteer_contract.to_string(),
            &lido_puppeteer_base::msg::QueryMsg::Extention {
                msg: lido_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?;

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
        FSM.go_to(deps.storage, ContractState::Claiming)?;
        if validators_to_claim.is_empty() {
            attrs.push(attr("validators_to_claim", "empty"));
            if let Some((transfer_msg, pending_amount)) =
                get_transfer_pending_balance(deps.as_ref(), &env, config, info.funds.clone())?
            {
                FSM.go_to(deps.storage, ContractState::Transfering)?;
                PENDING_TRANSFER.save(deps.storage, &pending_amount)?;
                messages.push(transfer_msg);
            } else {
                messages.push(get_stake_msg(deps.as_ref(), &env, config, info.funds)?);
                FSM.go_to(deps.storage, ContractState::Staking)?;
            }
        } else {
            attrs.push(attr("validators_to_claim", validators_to_claim.join(",")));
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.puppeteer_contract.to_string(),
            msg: to_json_binary(
                &lido_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                    validators: validators_to_claim,
                    transfer,
                    timeout: Some(config.puppeteer_timeout),
                    reply_to: env.contract.address.to_string(),
                },
            )?,
            funds: info.funds,
        }));
        }
        deps.api
            .debug(&format!("WASMDEBUG tick msg {:?}", messages));
        attrs.push(attr("state", "claiming"));
        LAST_IDLE_CALL.save(deps.storage, &env.block.time.seconds())?;
    }
    Ok(response("execute-tick_idle", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_claiming(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let response_msg = get_received_puppeteer_response(deps.as_ref())?;
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    let mut attrs = vec![attr("action", "tick_claiming")];
    let mut messages = vec![];
    match response_msg {
        lido_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) => {
            deps.api
                .debug(&format!("WASMDEBUG tick success {:?}", success_msg));
            match success_msg.transaction {
                lido_puppeteer_base::msg::Transaction::ClaimRewardsAndOptionalyTransfer {
                    transfer,
                    ..
                } => {
                    if let Some(transfer) = transfer {
                        let mut batch = UNBOND_BATCHES.load(deps.storage, transfer.batch_id)?;
                        batch.status = UnbondBatchStatus::Withdrawn;
                        attrs.push(attr("batch_id", transfer.batch_id.to_string()));
                        attrs.push(attr("unbond_batch_status", "withdrawn"));
                        UNBOND_BATCHES.save(deps.storage, transfer.batch_id, &batch)?;
                    }
                }
                _ => return Err(ContractError::InvalidTransaction {}),
            }
        }
        lido_puppeteer_base::msg::ResponseHookMsg::Error(err) => {
            deps.api.debug(&format!("WASMDEBUG tick error {:?}", err));
            attrs.push(attr("error_on_claiming", format!("{:?}", err)));
        }
    }
    if let Some((transfer_msg, pending_amount)) =
        get_transfer_pending_balance(deps.as_ref(), &env, config, info.funds.clone())?
    {
        FSM.go_to(deps.storage, ContractState::Transfering)?;
        PENDING_TRANSFER.save(deps.storage, &pending_amount)?;
        messages.push(transfer_msg);
    } else {
        messages.push(get_stake_msg(deps.as_ref(), &env, config, info.funds)?);
        FSM.go_to(deps.storage, ContractState::Staking)?;
    }
    attrs.push(attr("state", "unbonding"));
    Ok(response("execute-tick_claiming", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_tick_transfering(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let response_msg = get_received_puppeteer_response(deps.as_ref())?;
    deps.api
        .debug(&format!("WASMDEBUG tick transfering {:?}", response_msg));
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    let mut attrs = vec![attr("action", "tick_transfering")];
    FSM.go_to(deps.storage, ContractState::Staking)?;
    attrs.push(attr("state", "staking"));
    let message = get_stake_msg(deps.as_ref(), &env, config, info.funds)?;
    Ok(response("execute-tick_transfering", CONTRACT_NAME, attrs).add_message(message))
}

fn execute_tick_staking(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let response_msg = get_received_puppeteer_response(deps.as_ref())?;
    deps.api
        .debug(&format!("WASMDEBUG tick staking {:?}", response_msg));
    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    let mut attrs = vec![attr("action", "tick_staking")];
    let mut messages = vec![];
    let batch_id = FAILED_BATCH_ID
        .may_load(deps.storage)?
        .unwrap_or(UNBOND_BATCH_ID.load(deps.storage)?);
    let mut unbond = UNBOND_BATCHES.load(deps.storage, batch_id)?;
    if (Timestamp::from_seconds(unbond.created).plus_seconds(config.unbond_batch_switch_time)
        > env.block.time)
        && !unbond.unbond_items.is_empty()
        && !unbond.total_amount.is_zero()
    {
        deps.api.debug(&format!(
            "WASMDEBUG going to unbond id:{} {:?}",
            batch_id, unbond
        ));
        let (pre_unbonding_balance, _) = get_ica_balance_by_denom(
            deps.as_ref(),
            &config.puppeteer_contract,
            &config.remote_denom,
            true,
        )?;
        PRE_UNBONDING_BALANCE.save(deps.storage, &pre_unbonding_balance)?;
        FSM.go_to(deps.storage, ContractState::Unbonding)?;
        attrs.push(attr("state", "unbonding"));
        deps.api.debug(&format!(
            "WASMDEBUG query {:?} to {:?}",
            config.strategy_contract.to_string(),
            &lido_staking_base::msg::strategy::QueryMsg::CalcWithdraw {
                withdraw: unbond.total_amount,
            },
        ));
        let undelegations: Vec<lido_staking_base::msg::distribution::IdealDelegation> =
            deps.querier.query_wasm_smart(
                config.strategy_contract.to_string(),
                &lido_staking_base::msg::strategy::QueryMsg::CalcWithdraw {
                    withdraw: unbond.total_amount,
                },
            )?;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.puppeteer_contract.to_string(),
            msg: to_json_binary(&lido_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
                items: undelegations
                    .iter()
                    .map(|d| (d.valoper_address.clone(), d.stake_change))
                    .collect::<Vec<_>>(),
                batch_id,
                timeout: Some(config.puppeteer_timeout),
                reply_to: env.contract.address.to_string(),
            })?,
            funds: info.funds,
        }));
        unbond.status = UnbondBatchStatus::UnbondRequested;
        UNBOND_BATCHES.save(deps.storage, batch_id, &unbond)?;
        UNBOND_BATCH_ID.save(deps.storage, &(batch_id + 1))?;
        UNBOND_BATCHES.save(
            deps.storage,
            batch_id + 1,
            &new_unbond(env.block.time.seconds()),
        )?;
    } else {
        deps.api.debug("WASMDEBUG nothing to unbond");
        deps.api.debug(&format!(
            "WASMDEBUG goting from {:?} to {:?}",
            FSM.get_current_state(deps.storage)?,
            ContractState::Idle
        ));
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
        lido_puppeteer_base::msg::ResponseHookMsg::Success(response) => {
            match response.transaction {
                lido_puppeteer_base::msg::Transaction::Undelegate { batch_id, .. } => {
                    LAST_PUPPETEER_RESPONSE.remove(deps.storage);
                    attrs.push(attr("batch_id", batch_id.to_string()));
                    let mut unbond = UNBOND_BATCHES.load(deps.storage, batch_id)?;
                    unbond.status = UnbondBatchStatus::Unbonding;
                    unbond.expected_release = env.block.time.seconds() + config.unbonding_period;
                    UNBOND_BATCHES.save(deps.storage, batch_id, &unbond)?;
                    FAILED_BATCH_ID.remove(deps.storage);
                    attrs.push(attr("unbonding", "success"));
                }
                _ => return Err(ContractError::InvalidTransaction {}),
            }
        }
        lido_puppeteer_base::msg::ResponseHookMsg::Error(response) => match response.transaction {
            lido_puppeteer_base::msg::Transaction::Undelegate { batch_id, .. } => {
                attrs.push(attr("batch_id", batch_id.to_string()));
                let mut unbond = UNBOND_BATCHES.load(deps.storage, batch_id)?;
                unbond.status = UnbondBatchStatus::UnbondFailed;
                UNBOND_BATCHES.save(deps.storage, batch_id, &unbond)?;
                FAILED_BATCH_ID.save(deps.storage, &batch_id)?;
                attrs.push(attr("unbonding", "failed"));
            }
            _ => return Err(ContractError::InvalidTransaction {}),
        },
    }
    FSM.go_to(deps.storage, ContractState::Idle)?;
    Ok(response("execute-tick_unbonding", CONTRACT_NAME, attrs))
}

fn execute_fake_process_batch(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    batch_id: Uint128,
    unbonded_amount: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "fake_process_batch")];
    let mut unbond_batch = UNBOND_BATCHES.load(deps.storage, batch_id.into())?;
    unbond_batch.unbonded_amount = Some(unbonded_amount);
    unbond_batch.status = UnbondBatchStatus::Unbonded;
    unbond_batch.slashing_effect = Some(
        Decimal::from_str(&unbonded_amount.to_string())?
            / Decimal::from_str(&unbond_batch.expected_amount.to_string())?,
    );
    UNBOND_BATCHES.save(deps.storage, batch_id.into(), &unbond_batch)?;
    attrs.push(attr("batch_id", batch_id.to_string()));
    attrs.push(attr("unbonded_amount", unbonded_amount.to_string()));
    Ok(response("execute-fake_process_batch", CONTRACT_NAME, attrs))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let funds = info.funds;
    ensure_ne!(
        funds.len(),
        0,
        ContractError::InvalidFunds {
            reason: "no funds".to_string()
        }
    );
    ensure_eq!(
        funds.len(),
        1,
        ContractError::InvalidFunds {
            reason: "expected 1 denom".to_string()
        }
    );
    let mut attrs = vec![attr("action", "bond")];

    let amount = funds[0].amount;
    let denom = funds[0].denom.to_string();
    check_denom(denom)?;

    let exchange_rate = query_exchange_rate(deps.as_ref(), env, Some(amount))?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));

    let issue_amount = amount * (Decimal::one() / exchange_rate);
    attrs.push(attr("issue_amount", issue_amount.to_string()));

    let receiver = receiver.map_or(Ok::<String, ContractError>(info.sender.to_string()), |a| {
        deps.api.addr_validate(&a)?;
        Ok(a)
    })?;
    attrs.push(attr("receiver", receiver.clone()));

    let msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.token_contract,
        msg: to_json_binary(&TokenExecuteMsg::Mint {
            amount: issue_amount,
            receiver,
        })?,
        funds: vec![],
    })];
    Ok(response("execute-bond", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn check_denom(_denom: String) -> ContractResult<()> {
    //todo: check denom
    Ok(())
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    ensure_eq!(config.owner, info.sender, ContractError::Unauthorized {});
    let mut attrs = vec![attr("action", "update_config")];
    if let Some(token_contract) = new_config.token_contract {
        attrs.push(attr("token_contract", &token_contract));
        config.token_contract = token_contract;
    }
    if let Some(puppeteer_contract) = new_config.puppeteer_contract {
        attrs.push(attr("puppeteer_contract", &puppeteer_contract));
        config.puppeteer_contract = puppeteer_contract;
    }
    if let Some(puppeteer_timeout) = new_config.puppeteer_timeout {
        attrs.push(attr("puppeteer_contract", puppeteer_timeout.to_string()));
        config.puppeteer_timeout = puppeteer_timeout;
    }
    if let Some(strategy_contract) = new_config.strategy_contract {
        attrs.push(attr("strategy_contract", &strategy_contract));
        config.strategy_contract = strategy_contract;
    }
    if let Some(withdrawal_voucher_contract) = new_config.withdrawal_voucher_contract {
        attrs.push(attr(
            "withdrawal_voucher_contract",
            &withdrawal_voucher_contract,
        ));
        config.withdrawal_voucher_contract = withdrawal_voucher_contract;
    }
    if let Some(withdrawal_manager_contract) = new_config.withdrawal_manager_contract {
        attrs.push(attr(
            "withdrawal_manager_contract",
            &withdrawal_manager_contract,
        ));
        config.withdrawal_manager_contract = withdrawal_manager_contract;
    }
    if let Some(pump_address) = new_config.pump_address {
        attrs.push(attr("pump_address", &pump_address));
        config.pump_address = Some(pump_address);
    }
    if let Some(validators_set_contract) = new_config.validators_set_contract {
        attrs.push(attr("validators_set_contract", &validators_set_contract));
        config.validators_set_contract = validators_set_contract;
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
    if let Some(unbond_batch_switch_time) = new_config.unbond_batch_switch_time {
        attrs.push(attr(
            "unbond_batch_switch_time",
            unbond_batch_switch_time.to_string(),
        ));
        config.unbond_batch_switch_time = unbond_batch_switch_time;
    }
    if let Some(owner) = new_config.owner {
        attrs.push(attr("owner", &owner));
        config.owner = owner;
    }
    if let Some(ld_denom) = new_config.ld_denom {
        attrs.push(attr("ld_denom", &ld_denom));
        config.ld_denom = Some(ld_denom);
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(response("execute-update_config", CONTRACT_NAME, attrs))
}

fn execute_unbond(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "unbond")];
    let unbond_batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    ensure_eq!(
        info.funds.len(),
        1,
        ContractError::InvalidFunds {
            reason: "Must be one token".to_string(),
        }
    );
    let config = CONFIG.load(deps.storage)?;
    let ld_denom = config.ld_denom.ok_or(ContractError::LDDenomIsNotSet {})?;
    let amount = info.funds[0].amount;
    let denom = info.funds[0].denom.to_string();
    ensure_eq!(
        denom,
        ld_denom,
        ContractError::InvalidFunds {
            reason: "Must be LD token".to_string(),
        }
    );
    let mut unbond_batch = UNBOND_BATCHES.load(deps.storage, unbond_batch_id)?;
    let exchange_rate = query_exchange_rate(deps.as_ref(), env, None)?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));
    let expected_amount = amount * exchange_rate;
    unbond_batch.unbond_items.push(UnbondItem {
        sender: info.sender.to_string(),
        amount,
        expected_amount,
    });
    unbond_batch.total_amount += amount;
    unbond_batch.expected_amount += expected_amount;

    attrs.push(attr("expected_amount", expected_amount.to_string()));
    UNBOND_BATCHES.save(deps.storage, unbond_batch_id, &unbond_batch)?;
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
            contract_addr: config.withdrawal_voucher_contract,
            msg: to_json_binary(&VoucherExecuteMsg::Mint {
                owner: info.sender.to_string(),
                token_id: unbond_batch_id.to_string()
                    + "_"
                    + info.sender.to_string().as_str()
                    + "_"
                    + &unbond_batch.unbond_items.len().to_string(),
                token_uri: None,
                extension,
            })?,
            funds: vec![],
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.token_contract,
            msg: to_json_binary(&TokenExecuteMsg::Burn {})?,
            funds: vec![cosmwasm_std::Coin {
                denom: ld_denom,
                amount,
            }],
        }),
    ];
    Ok(response("execute-unbond", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn get_unbonded_batch(deps: Deps<NeutronQuery>) -> ContractResult<Option<(u128, UnbondBatch)>> {
    let batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    if batch_id == 0 {
        return Ok(None);
    }
    let batch = UNBOND_BATCHES.load(deps.storage, batch_id - 1)?;
    if batch.status == UnbondBatchStatus::Unbonded {
        return Ok(Some((batch_id - 1, batch)));
    }
    Ok(None)
}

fn get_transfer_pending_balance<T>(
    deps: Deps<NeutronQuery>,
    env: &Env,
    config: &Config,
    funds: Vec<cosmwasm_std::Coin>,
) -> ContractResult<Option<(CosmosMsg<T>, Uint128)>> {
    let pending_amount = deps
        .querier
        .query_balance(
            env.contract.address.to_string(),
            config.base_denom.to_string(),
        )?
        .amount;
    if pending_amount.is_zero() {
        return Ok(None);
    }
    let mut all_funds = vec![cosmwasm_std::Coin {
        denom: config.base_denom.to_string(),
        amount: pending_amount,
    }];
    all_funds.extend(funds);

    Ok(Some((
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.puppeteer_contract.to_string(),
            msg: to_json_binary(
                &lido_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
                    timeout: config.puppeteer_timeout,
                    reply_to: env.contract.address.to_string(),
                },
            )?,
            funds: all_funds,
        }),
        pending_amount,
    )))
}

fn get_stake_msg<T>(
    deps: Deps<NeutronQuery>,
    env: &Env,
    config: &Config,
    funds: Vec<cosmwasm_std::Coin>,
) -> ContractResult<CosmosMsg<T>> {
    let (balance, balance_height) = get_ica_balance_by_denom(
        deps,
        &config.puppeteer_contract,
        &config.remote_denom,
        false,
    )?;
    ensure_ne!(balance, Uint128::zero(), ContractError::ICABalanceZero {});
    let last_ica_balance_change_height = LAST_ICA_BALANCE_CHANGE_HEIGHT.load(deps.storage)?;
    ensure!(
        last_ica_balance_change_height <= balance_height,
        ContractError::PuppeteerBalanceOutdated {
            ica_height: last_ica_balance_change_height,
            puppeteer_height: balance_height
        }
    );
    let to_delegate: Vec<lido_staking_base::msg::distribution::IdealDelegation> =
        deps.querier.query_wasm_smart(
            &config.strategy_contract,
            &lido_staking_base::msg::strategy::QueryMsg::CalcDeposit { deposit: balance },
        )?;
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract.to_string(),
        msg: to_json_binary(&lido_staking_base::msg::puppeteer::ExecuteMsg::Delegate {
            items: to_delegate
                .iter()
                .map(|d| (d.valoper_address.to_string(), d.stake_change))
                .collect::<Vec<_>>(),
            timeout: Some(config.puppeteer_timeout),
            reply_to: env.contract.address.to_string(),
        })?,
        funds,
    }))
}

fn get_received_puppeteer_response(
    deps: Deps<NeutronQuery>,
) -> ContractResult<lido_puppeteer_base::msg::ResponseHookMsg> {
    LAST_PUPPETEER_RESPONSE
        .load(deps.storage)
        .map_err(|_| ContractError::PuppeteerResponseIsNotReceived {})
}

fn is_unbonding_time_close(
    deps: Deps<NeutronQuery>,
    now: &u64,
    safe_period: &u64,
) -> ContractResult<bool> {
    let mut unbond_batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    while unbond_batch_id > 0 {
        let unbond_batch = UNBOND_BATCHES.load(deps.storage, unbond_batch_id)?;
        if unbond_batch.status == UnbondBatchStatus::Unbonding
            && (now - unbond_batch.expected_release < *safe_period)
        {
            return Ok(true);
        }
        if unbond_batch.status == UnbondBatchStatus::Unbonded {
            return Ok(false);
        }
        unbond_batch_id -= 1;
    }
    Ok(false)
}

fn get_ica_balance_by_denom<T: CustomQuery>(
    deps: Deps<T>,
    puppeteer_contract: &str,
    remote_denom: &str,
    can_be_zero: bool,
) -> ContractResult<(Uint128, u64)> {
    let (ica_balances, remote_height): lido_staking_base::msg::puppeteer::BalancesResponse =
        deps.querier.query_wasm_smart(
            puppeteer_contract.to_string(),
            &lido_puppeteer_base::msg::QueryMsg::Extention {
                msg: lido_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )?;
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
        remote_height,
    ))
}

fn new_unbond(now: u64) -> lido_staking_base::state::core::UnbondBatch {
    lido_staking_base::state::core::UnbondBatch {
        total_amount: Uint128::zero(),
        expected_amount: Uint128::zero(),
        unbond_items: vec![],
        status: UnbondBatchStatus::New,
        expected_release: 0,
        slashing_effect: None,
        unbonded_amount: None,
        withdrawed_amount: None,
        created: now,
    }
}

fn get_non_native_rewards_transfer_msg<T>(
    deps: Deps<NeutronQuery>,
    info: MessageInfo,
    env: Env,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let config = CONFIG.load(deps.storage)?;
    let non_native_rewards_receivers = NON_NATIVE_REWARDS_CONFIG.load(deps.storage)?;
    let mut items = vec![];
    let rewards: lido_staking_base::msg::puppeteer::BalancesResponse =
        deps.querier.query_wasm_smart(
            config
                .pump_address
                .ok_or(ContractError::PumpAddressIsNotSet {})?,
            &lido_puppeteer_base::msg::QueryMsg::Extention {
                msg: lido_staking_base::msg::puppeteer::QueryExtMsg::NonNativeRewardsBalances {},
            },
        )?;
    let rewards_map = rewards
        .0
        .coins
        .iter()
        .map(|c| (c.denom.clone(), c.amount))
        .collect::<std::collections::HashMap<_, _>>();
    for item in non_native_rewards_receivers {
        let amount =
            rewards_map
                .get(&item.denom)
                .ok_or(ContractError::NonNativeRewardsDenomNotFound {
                    denom: item.denom.clone(),
                })?;
        if amount > &item.min_amount {
            items.push((
                item.address,
                cosmwasm_std::Coin {
                    denom: item.denom,
                    amount: *amount,
                },
            ));
        }
    }
    if items.is_empty() {
        return Ok(None);
    }
    Ok(Some(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract,
        msg: to_json_binary(&lido_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
            items,
            timeout: Some(config.puppeteer_timeout),
            reply_to: env.contract.address.to_string(),
        })?,
        funds: info.funds,
    })))
}
