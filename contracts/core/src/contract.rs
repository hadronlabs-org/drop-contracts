use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{
    attr, ensure, ensure_eq, ensure_ne, entry_point, to_json_binary, Attribute, Binary, CosmosMsg,
    Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use lido_helpers::{answer::response, fsm::Fsm};
use lido_puppeteer_base::msg::TransferReadyBatchMsg;
use lido_staking_base::state::core::{
    get_transitions, Config, ContractState, UnbondBatch, UnbondBatchStatus, UnbondItem, CONFIG,
    FSM, UNBOND_BATCHES, UNBOND_BATCH_ID,
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
    _env: Env,
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
    UNBOND_BATCHES.save(
        deps.storage,
        0,
        &lido_staking_base::state::core::UnbondBatch {
            total_amount: Uint128::zero(),
            expected_amount: Uint128::zero(),
            unbond_items: vec![],
            status: UnbondBatchStatus::New,
            expected_release: 0,
            slashing_effect: None,
            unbonded_amount: None,
            withdrawed_amount: None,
        },
    )?;
    FSM.save(
        deps.storage,
        &Fsm::new(ContractState::Idle, get_transitions()),
    )?;
    LAST_IDLE_CALL.save(deps.storage, &0);
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::ExchangeRate {} => to_json_binary(&query_exchange_rate(deps, env)?),
        QueryMsg::UnbondBatch { batch_id } => query_unbond_batch(deps, batch_id),
    }
}

fn query_exchange_rate(_deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Decimal> {
    Decimal::from_str("1.01")
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
        ExecuteMsg::UpdateConfig {
            token_contract,
            puppeteer_contract,
            strategy_contract,
            owner,
            ld_denom,
            tick_min_interval,
        } => execute_update_config(
            deps,
            info,
            token_contract,
            puppeteer_contract,
            strategy_contract,
            owner,
            ld_denom,
            tick_min_interval,
        ),
        ExecuteMsg::FakeProcessBatch {
            batch_id,
            unbonded_amount,
        } => execute_fake_process_batch(deps, env, info, batch_id, unbonded_amount),
        ExecuteMsg::Tick {} => execute_tick(deps, env, info, None),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
    }
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: lido_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    execute_tick(deps, env, info, Some(msg))
}

fn execute_tick(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    response_msg: Option<lido_puppeteer_base::msg::ResponseHookMsg>,
) -> ContractResult<Response<NeutronMsg>> {
    let mut machine = FSM.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    match machine.current_state {
        ContractState::Idle => execute_tick_idle(deps, env, info, &machine, &config),
        ContractState::Claiming => {
            execute_tick_claiming(deps, env, info, &machine, &config, response_msg)
        }
        ContractState::Unbonding => execute_tick_unbonding(deps, env, info),
        ContractState::Staking => execute_tick_staking(deps, env, info),
    }
    .map(|r| {
        LAST_IDLE_CALL.save(deps.storage, &env.block.time.seconds())?;
        r
    })
}

fn execute_tick_idle(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    machine: &Fsm<ContractState>,
    config: &Config,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "tick_idle")];
    let last_idle_call = LAST_IDLE_CALL.load(deps.storage)?;
    let idle_min_interval = config.idle_min_interval;
    let pump_address = config
        .pump_address
        .ok_or(ContractError::PumpAddressIsNotSet {})?;
    ensure!(
        env.block.time.seconds() - last_idle_call >= idle_min_interval,
        ContractError::IdleMinIntervalIsNotReached {}
    );
    ensure!(
        !is_unbonding_time_close(
            deps.as_ref(),
            &env.block.time.seconds(),
            &config.unbonding_safe_period
        )?,
        ContractError::UnbondingTimeIsClose {}
    );
    ensure!(
        env.block.time.seconds() - last_idle_call >= idle_min_interval,
        ContractError::IdleMinIntervalIsNotReached {}
    );
    ensure!(
        !is_unbonding_time_close(
            deps.as_ref(),
            &env.block.time.seconds(),
            &config.unbonding_safe_period
        )?,
        ContractError::UnbondingTimeIsClose {}
    );
    // process unbond if any aleready unbonded
    // and claim rewards
    let transfer =
        get_unbonded_batch(deps.as_ref())?.map(|(batch_id, batch)| TransferReadyBatchMsg {
            batch_id,
            amount: batch.unbonded_amount.unwrap(),
            recipient: pump_address,
        });

    let validators: Vec<ValidatorInfo> = deps.querier.query_wasm_smart(
        config.validator_set_contract,
        &lido_staking_base::msg::validatorset::QueryMsg::Validators {},
    )?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract,
        msg: to_json_binary(
            &lido_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
                validators: validators.iter().map(|v| v.valoper_address).collect(),
                transfer,
                timeout: Some(config.puppeteer_timeout),
                reply_to: env.contract.address.to_string(),
            },
        )?,
        funds: vec![],
    });

    machine.go_to(ContractState::Claiming)?;
    FSM.save(deps.storage, &machine)?;
    attrs.push(attr("state", "claiming"));
    Ok(response("execute-tick_idle", CONTRACT_NAME, attrs).add_message(msg))
}

fn execute_tick_claiming(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    machine: &Fsm<ContractState>,
    config: &Config,
    response_msg: Option<lido_puppeteer_base::msg::ResponseHookMsg>,
) -> ContractResult<Response<NeutronMsg>> {
    let response_msg = response_msg.ok_or(ContractError::ResponseIsEmpty {})?;
    let mut attrs = vec![attr("action", "tick_claiming")];
    match response_msg {
        lido_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) => {
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
            machine.go_to(ContractState::Staking)?;
            FSM.save(deps.storage, &machine)?;
            attrs.push(attr("state", "unbonding"));
            Ok(response("execute-tick_claiming", CONTRACT_NAME, attrs))
        }
        lido_puppeteer_base::msg::ResponseHookMsg::Error(err) => {
            machine.go_to(ContractState::Idle)?;
            FSM.save(deps.storage, &machine)?;
            attrs.push(attr("error_on_claiming", format!("{:?}", err)));
            Ok(response("execute-tick_claiming", CONTRACT_NAME, attrs))
        }
    }
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

    let exchange_rate = query_exchange_rate(deps.as_ref(), env)?;
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
    token_contract: Option<String>,
    puppeteer_contract: Option<String>,
    strategy_contract: Option<String>,
    owner: Option<String>,
    ld_denom: Option<String>,
    tick_min_interval: Option<u64>,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    ensure_eq!(config.owner, info.sender, ContractError::Unauthorized {});

    let mut attrs = vec![attr("action", "update_config")];
    if let Some(token_contract) = token_contract {
        attrs.push(attr("token_contract", &token_contract));
        config.token_contract = token_contract;
    }
    if let Some(puppeteer_contract) = puppeteer_contract {
        attrs.push(attr("puppeteer_contract", &puppeteer_contract));
        config.puppeteer_contract = puppeteer_contract;
    }
    if let Some(strategy_contract) = strategy_contract {
        attrs.push(attr("strategy_contract", &strategy_contract));
        config.strategy_contract = strategy_contract;
    }
    if let Some(owner) = owner {
        attrs.push(attr("owner", &owner));
        config.owner = owner;
    }
    if let Some(ld_denom) = ld_denom {
        attrs.push(attr("ld_denom", &ld_denom));
        config.ld_denom = Some(ld_denom);
    }
    if let Some(tick_min_interval) = tick_min_interval {
        attrs.push(attr("tick_min_interval", tick_min_interval.to_string()));
        config.idle_min_interval = tick_min_interval;
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
    let exchange_rate = query_exchange_rate(deps.as_ref(), env)?;
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
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
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
    });

    Ok(response("execute-unbond", CONTRACT_NAME, attrs).add_message(msg))
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
