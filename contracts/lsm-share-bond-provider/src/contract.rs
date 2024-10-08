use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Addr, Attribute, Coin, CosmosMsg, CustomQuery,
    Decimal, Decimal256, Deps, Empty, Reply, StdError, StdResult, SubMsg, SubMsgResult, Uint128,
    Uint256, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_helpers::ibc_client_state::query_client_state;
use drop_helpers::ibc_fee::query_ibc_fee;
use drop_puppeteer_base::peripheral_hook::{
    IBCTransferReason, ReceiverExecuteMsg, ResponseAnswer, ResponseHookErrorMsg, ResponseHookMsg,
    ResponseHookSuccessMsg, Transaction,
};
use drop_puppeteer_base::proto::MsgIBCTransfer;
use drop_puppeteer_base::state::RedeemShareItem;
use drop_staking_base::error::lsm_share_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::core::LastPuppeteerResponse;
use drop_staking_base::msg::lsm_share_bond_provider::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::core::LAST_PUPPETEER_RESPONSE;
use drop_staking_base::state::lsm_share_bond_provider::{
    Config, ConfigOptional, ReplyMsg, TxState, TxStateStatus, CONFIG, LAST_LSM_REDEEM,
    LSM_SHARES_TO_REDEEM, PENDING_LSM_SHARES, TOTAL_LSM_SHARES, TX_STATE,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::interchain_queries::v047::types::DECIMAL_FRACTIONAL;
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, SudoMsg};
use prost::Message;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const LOCAL_DENOM: &str = "untrn";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let puppeteer_contract = deps.api.addr_validate(&msg.puppeteer_contract)?;
    let core_contract = deps.api.addr_validate(&msg.core_contract)?;
    let validators_set_contract = deps.api.addr_validate(&msg.validators_set_contract)?;
    let config = &Config {
        puppeteer_contract: puppeteer_contract.clone(),
        core_contract: core_contract.clone(),
        validators_set_contract: validators_set_contract.clone(),
        port_id: msg.port_id.to_string(),
        transfer_channel_id: msg.transfer_channel_id.to_string(),
        timeout: msg.timeout,
        lsm_min_bond_amount: msg.lsm_min_bond_amount,
        lsm_redeem_threshold: msg.lsm_redeem_threshold,
        lsm_redeem_maximum_interval: msg.lsm_redeem_maximum_interval,
    };
    CONFIG.save(deps.storage, config)?;

    TOTAL_LSM_SHARES.save(deps.storage, &0)?;
    LAST_LSM_REDEEM.save(deps.storage, &env.block.time.seconds())?;
    TX_STATE.save(deps.storage, &TxState::default())?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("puppeteer_contract", puppeteer_contract),
            attr("core_contract", core_contract),
            attr("validators_set_contract", validators_set_contract),
            attr("port_id", msg.port_id),
            attr("transfer_channel_id", msg.transfer_channel_id),
            attr("timeout", msg.timeout.to_string()),
            attr("lsm_min_bond_amount", msg.lsm_min_bond_amount.to_string()),
            attr("lsm_redeem_threshold", msg.lsm_redeem_threshold.to_string()),
            attr(
                "lsm_redeem_maximum_interval",
                msg.lsm_redeem_maximum_interval.to_string(),
            ),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::CanBond { denom } => query_can_bond(deps, denom),
        QueryMsg::CanProcessOnIdle {} => query_can_process_on_idle(deps, env),
        QueryMsg::TokensAmount {
            coin,
            exchange_rate,
        } => query_token_amount(deps, coin, exchange_rate),
        QueryMsg::PendingLSMShares {} => query_pending_lsm_shares(deps),
        QueryMsg::LSMSharesToRedeem {} => query_lsm_shares_to_redeem(deps),
        QueryMsg::LastPuppeteerResponse {} => to_json_binary(&LastPuppeteerResponse {
            response: LAST_PUPPETEER_RESPONSE.may_load(deps.storage)?,
        })
        .map_err(From::from),
        QueryMsg::TxState {} => query_tx_state(deps, env),
        QueryMsg::AsyncTokensAmount {} => {
            to_json_binary(&TOTAL_LSM_SHARES.load(deps.storage)?).map_err(From::from)
        }
    }
}

fn query_tx_state(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let tx_state = TX_STATE.load(deps.storage)?;
    Ok(to_json_binary(&tx_state)?)
}

fn query_pending_lsm_shares(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let shares: Vec<(String, (String, Uint128, Uint128))> = PENDING_LSM_SHARES
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    to_json_binary(&shares).map_err(From::from)
}

fn query_lsm_shares_to_redeem(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let shares: Vec<(String, (String, Uint128, Uint128))> = LSM_SHARES_TO_REDEEM
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    to_json_binary(&shares).map_err(From::from)
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_can_bond(deps: Deps<NeutronQuery>, denom: String) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let check_denom_result = check_denom::check_denom(&deps, &denom, &config);

    Ok(to_json_binary(&check_denom_result.is_ok())?)
}

fn query_can_process_on_idle(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::Idle,
        ContractError::InvalidState {
            reason: "tx_state is not idle".to_string()
        }
    );

    let config = CONFIG.load(deps.storage)?;

    let pending_lsm_shares_count = PENDING_LSM_SHARES
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .count();

    if pending_lsm_shares_count > 0 {
        return Ok(to_json_binary(&true)?);
    }

    let lsm_shares_to_redeem_count = LSM_SHARES_TO_REDEEM
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .count();

    let last_lsm_redeem = LAST_LSM_REDEEM.load(deps.storage)?;
    let lsm_redeem_threshold = config.lsm_redeem_threshold as usize;

    if pending_lsm_shares_count == 0 && lsm_shares_to_redeem_count == 0 {
        return Ok(to_json_binary(&false)?);
    }

    if lsm_shares_to_redeem_count >= lsm_redeem_threshold
        || (last_lsm_redeem + config.lsm_redeem_maximum_interval < env.block.time.seconds())
    {
        return Ok(to_json_binary(&true)?);
    }

    Ok(to_json_binary(&false)?)
}

fn query_token_amount(
    deps: Deps<NeutronQuery>,
    coin: Coin,
    exchange_rate: Decimal,
) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    let check_denom = check_denom::check_denom(&deps, &coin.denom, &config)?;

    let real_amount = calc_lsm_share_underlying_amount(
        deps,
        &config.puppeteer_contract,
        &coin.amount,
        check_denom.validator,
    )?;

    let issue_amount = real_amount * (Decimal::one() / exchange_rate);

    Ok(to_json_binary(&issue_amount)?)
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
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::Bond {} => execute_bond(deps, info),
        ExecuteMsg::ProcessOnIdle {} => execute_process_on_idle(deps, env),
        ExecuteMsg::PeripheralHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
    }
}

fn execute_process_on_idle(
    mut deps: DepsMut<NeutronQuery>,
    env: Env,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api
        .debug("WASMDEBUG: lsm-share execute_process_on_idle: 1");
    let config = CONFIG.load(deps.storage)?;

    let mut attrs = vec![attr("action", "process_on_idle")];
    let mut submessages: Vec<SubMsg<NeutronMsg>> = vec![];
    deps.api
        .debug("WASMDEBUG: lsm-share  execute_process_on_idle: 2");

    attrs.push(attr("knot", "036"));
    if let Some(lsm_msg) = get_pending_redeem_msg(deps.branch(), &config, &env)? {
        deps.api
            .debug("WASMDEBUG: lsm-share  execute_process_on_idle: 3");
        submessages.push(lsm_msg);
        attrs.push(attr("knot", "037"));
        attrs.push(attr("knot", "038"));
    } else {
        attrs.push(attr("knot", "041"));
        if let Some(lsm_msg) = get_pending_lsm_share_msg(deps.branch(), &config, &env)? {
            deps.api
                .debug("WASMDEBUG: lsm-share  execute_process_on_idle: 4");
            submessages.push(lsm_msg);
            attrs.push(attr("knot", "042"));
            attrs.push(attr("knot", "043"));
        }
    }

    deps.api
        .debug("WASMDEBUG: lsm-share  execute_process_on_idle: 6");

    Ok(
        response("update_config", CONTRACT_NAME, Vec::<Attribute>::new())
            .add_submessages(submessages)
            .add_attributes(attrs),
    )
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut state = CONFIG.load(deps.storage)?;
    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(puppeteer_contract) = new_config.puppeteer_contract {
        state.puppeteer_contract = deps.api.addr_validate(puppeteer_contract.as_ref())?;
        attrs.push(attr("puppeteer_contract", puppeteer_contract))
    }

    if let Some(core_contract) = new_config.core_contract {
        state.core_contract = deps.api.addr_validate(core_contract.as_ref())?;
        attrs.push(attr("core_contract", core_contract))
    }

    if let Some(validators_set_contract) = new_config.validators_set_contract {
        state.validators_set_contract = deps.api.addr_validate(validators_set_contract.as_ref())?;
        attrs.push(attr("validators_set_contract", validators_set_contract))
    }

    if let Some(port_id) = new_config.port_id {
        state.port_id = port_id.to_string();
        attrs.push(attr("port_id", port_id))
    }

    if let Some(transfer_channel_id) = new_config.transfer_channel_id {
        state.transfer_channel_id = transfer_channel_id.to_string();
        attrs.push(attr("transfer_channel_id", transfer_channel_id))
    }

    if let Some(timeout) = new_config.timeout {
        state.timeout = timeout;
        attrs.push(attr("timeout", timeout.to_string()))
    }

    if let Some(lsm_min_bond_amount) = new_config.lsm_min_bond_amount {
        state.lsm_min_bond_amount = lsm_min_bond_amount;
        attrs.push(attr("lsm_min_bond_amount", lsm_min_bond_amount.to_string()))
    }

    if let Some(lsm_redeem_threshold) = new_config.lsm_redeem_threshold {
        state.lsm_redeem_threshold = lsm_redeem_threshold;
        attrs.push(attr(
            "lsm_redeem_threshold",
            lsm_redeem_threshold.to_string(),
        ))
    }

    if let Some(lsm_redeem_maximum_interval) = new_config.lsm_redeem_maximum_interval {
        state.lsm_redeem_maximum_interval = lsm_redeem_maximum_interval;
        attrs.push(attr(
            "lsm_redeem_maximum_interval",
            lsm_redeem_maximum_interval.to_string(),
        ))
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let Coin { amount, denom } = cw_utils::one_coin(&info)?;
    let config = CONFIG.load(deps.storage)?;

    let check_denom = check_denom::check_denom(&deps.as_ref(), &denom, &config)?;

    let real_amount = calc_lsm_share_underlying_amount(
        deps.as_ref(),
        &config.puppeteer_contract,
        &amount,
        check_denom.validator,
    )?;

    if real_amount < config.lsm_min_bond_amount {
        return Err(ContractError::LSMBondAmountIsBelowMinimum {
            min_stake_amount: config.lsm_min_bond_amount,
            bond_amount: real_amount,
        });
    }

    TOTAL_LSM_SHARES.update(deps.storage, |total| {
        StdResult::Ok(total + real_amount.u128())
    })?;
    PENDING_LSM_SHARES.update(deps.storage, denom.to_string(), |one| {
        let mut new = one.unwrap_or((
            check_denom.remote_denom.to_string(),
            Uint128::zero(),
            Uint128::zero(),
        ));
        new.1 += amount;
        new.2 += real_amount;
        StdResult::Ok(new)
    })?;

    Ok(response(
        "bond",
        CONTRACT_NAME,
        [
            attr_coin("received_funds", amount.to_string(), denom),
            attr_coin(
                "bonded_funds",
                real_amount.to_string(),
                check_denom.remote_denom,
            ),
        ],
    ))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::peripheral_hook::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );
    if let drop_puppeteer_base::peripheral_hook::ResponseHookMsg::Success(success_msg) = msg.clone()
    {
        if let drop_puppeteer_base::peripheral_hook::Transaction::RedeemShares { items, .. } =
            &success_msg.transaction
        {
            let mut sum = 0u128;
            for item in items {
                let (_remote_denom, _shares_amount, real_amount) =
                    LSM_SHARES_TO_REDEEM.load(deps.storage, item.local_denom.to_string())?;
                sum += real_amount.u128();
                LSM_SHARES_TO_REDEEM.remove(deps.storage, item.local_denom.to_string());
            }
            TOTAL_LSM_SHARES.update(deps.storage, |one| StdResult::Ok(one - sum))?;
            LAST_LSM_REDEEM.save(deps.storage, &env.block.time.seconds())?;
        }
    }

    LAST_PUPPETEER_RESPONSE.save(deps.storage, &msg)?;

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(msg))?,
        funds: vec![],
    });

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    )
    .add_message(hook_message))
}

pub fn get_pending_redeem_msg(
    deps: DepsMut<NeutronQuery>,
    config: &Config,
    env: &Env,
) -> ContractResult<Option<SubMsg<NeutronMsg>>> {
    let pending_lsm_shares_count = LSM_SHARES_TO_REDEEM
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .count();
    let last_lsm_redeem = LAST_LSM_REDEEM.load(deps.storage)?;
    let lsm_redeem_threshold = config.lsm_redeem_threshold as usize;

    if pending_lsm_shares_count == 0
        || ((pending_lsm_shares_count < lsm_redeem_threshold)
            && (last_lsm_redeem + config.lsm_redeem_maximum_interval > env.block.time.seconds()))
    {
        return Ok(None);
    }
    let shares_to_redeeem = LSM_SHARES_TO_REDEEM
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .take(lsm_redeem_threshold)
        .collect::<StdResult<Vec<_>>>()?;

    let items: Vec<RedeemShareItem> = shares_to_redeeem
        .iter()
        .map(
            |(local_denom, (denom, share_amount, _real_amount))| RedeemShareItem {
                amount: *share_amount,
                local_denom: local_denom.to_string(),
                remote_denom: denom.to_string(),
            },
        )
        .collect();

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.puppeteer_contract.to_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
                items: items.clone(),
                reply_to: env.contract.address.to_string(),
            },
        )?,
        funds: vec![],
    });

    let submsg = msg_with_reply_callback(
        deps,
        msg,
        Transaction::RedeemShares { items },
        ReplyMsg::Redeem.to_reply_id(),
    )?;

    Ok(Some(submsg))
}

fn get_pending_lsm_share_msg(
    deps: DepsMut<NeutronQuery>,
    config: &Config,
    env: &Env,
) -> ContractResult<Option<SubMsg<NeutronMsg>>> {
    let lsm_share: Option<(String, (String, Uint128, Uint128))> =
        PENDING_LSM_SHARES.first(deps.storage)?;
    match lsm_share {
        Some((local_denom, (_remote_denom, share_amount, real_amount))) => {
            let puppeteer_ica: drop_helpers::ica::IcaState = deps.querier.query_wasm_smart(
                &config.puppeteer_contract,
                &drop_puppeteer_base::msg::QueryMsg::<Empty>::Ica {},
            )?;

            if let drop_helpers::ica::IcaState::Registered { ica_address, .. } = puppeteer_ica {
                let pending_token = Coin::new(share_amount.u128(), local_denom.clone());

                let msg = NeutronMsg::IbcTransfer {
                    source_port: config.port_id.clone(),
                    source_channel: config.transfer_channel_id.clone(),
                    token: pending_token.clone(),
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

                let submsg = msg_with_reply_callback(
                    deps,
                    msg,
                    Transaction::IBCTransfer {
                        real_amount: real_amount.u128(),
                        denom: local_denom,
                        amount: share_amount.u128(),
                        recipient: ica_address.to_string(),
                        reason: IBCTransferReason::LSMShare,
                    },
                    ReplyMsg::IbcTransfer.to_reply_id(),
                )?;

                Ok(Some(submsg))
            } else {
                Err(ContractError::IcaNotRegistered {})
            }
        }
        None => Ok(None),
    }
}

fn msg_with_reply_callback<C: Into<CosmosMsg<X>> + Serialize, X>(
    deps: DepsMut<NeutronQuery>,
    msg: C,
    transaction: Transaction,
    payload_id: u64,
) -> StdResult<SubMsg<X>> {
    TX_STATE.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::InProgress,
            transaction: Some(transaction),
        },
    )?;
    Ok(SubMsg::reply_always(msg, payload_id))
}

fn calc_lsm_share_underlying_amount<T: CustomQuery>(
    deps: Deps<T>,
    puppeteer_contract: &Addr,
    lsm_share: &Uint128,
    validator: String,
) -> ContractResult<Uint128> {
    let delegations = deps
        .querier
        .query_wasm_smart::<drop_staking_base::msg::puppeteer::DelegationsResponse>(
            puppeteer_contract,
            &drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )?
        .delegations
        .delegations;
    if delegations.is_empty() {
        return Err(ContractError::NoDelegations {});
    }
    let validator_info = delegations
        .iter()
        .find(|one| one.validator == validator)
        .ok_or(ContractError::ValidatorInfoNotFound {
            validator: validator.clone(),
        })?;
    let share = Decimal256::from_atomics(*lsm_share, 0)?;
    Ok(Uint128::try_from(
        share.checked_mul(validator_info.share_ratio)?.atomics()
            / Uint256::from(DECIMAL_FRACTIONAL),
    )?)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    if let SubMsgResult::Err(err) = msg.result {
        return Err(ContractError::PuppeteerError { message: err });
    }

    match ReplyMsg::from_reply_id(msg.id) {
        ReplyMsg::IbcTransfer | ReplyMsg::Redeem => puppeteer_reply(deps),
    }
}

fn puppeteer_reply(deps: DepsMut) -> ContractResult<Response> {
    let mut tx_state: TxState = TX_STATE.load(deps.storage)?;
    tx_state.status = TxStateStatus::WaitingForAck;
    TX_STATE.save(deps.storage, &tx_state)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: SudoMsg,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?} block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_error(deps, env, request, "Timeout".to_string()),
        _ => Err(ContractError::MessageIsNotSupported {}),
    }
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    details: String,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_error: request: {request:?}",
        request = request
    ));

    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::WaitingForAck,
        ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    );

    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let attrs = vec![
        attr("action", "sudo_error"),
        attr("request_id", seq_id.to_string()),
    ];

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;

    TX_STATE.save(deps.storage, &TxState::default())?;

    deps.api.debug(&format!(
        "WASMDEBUG: sudo_timeout: request: {request:?}",
        request = request
    ));

    let config = CONFIG.load(deps.storage)?;

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(ResponseHookMsg::Error(
            ResponseHookErrorMsg {
                request_id: seq_id,
                request,
                transaction,
                details,
            },
        )))?,
        funds: vec![],
    });

    Ok(response("sudo-timeout", "puppeteer", attrs).add_message(hook_message))
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    request: RequestPacket,
    _data: Binary,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.debug("WASMDEBUG: sudo response");

    let tx_state = TX_STATE.load(deps.storage)?;
    ensure!(
        tx_state.status == TxStateStatus::WaitingForAck,
        ContractError::InvalidState {
            reason: "tx_state is not WaitingForAck".to_string()
        }
    );

    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let transaction = tx_state
        .transaction
        .ok_or_else(|| StdError::generic_err("transaction not found"))?;

    let channel_id = request
        .clone()
        .source_channel
        .ok_or_else(|| StdError::generic_err("source_channel not found"))?;
    let port_id = request
        .clone()
        .source_port
        .ok_or_else(|| StdError::generic_err("source_port not found"))?;

    let client_state = query_client_state(&deps.as_ref(), channel_id, port_id)?;

    let remote_height = client_state
        .identified_client_state
        .ok_or_else(|| StdError::generic_err("IBC client state identified_client_state not found"))?
        .client_state
        .latest_height
        .ok_or_else(|| StdError::generic_err("IBC client state latest_height not found"))?
        .revision_height;

    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];

    if let Transaction::IBCTransfer {
        amount,
        denom,
        real_amount,
        ..
    } = transaction.clone()
    {
        let current_pending = PENDING_LSM_SHARES.may_load(deps.storage, denom.to_string())?;
        if let Some((remote_denom, shares_amount, _real_amount)) = current_pending {
            let sent_amount = Uint128::from(amount);
            let sent_real_amount = Uint128::from(real_amount);

            LSM_SHARES_TO_REDEEM.update(deps.storage, denom.to_string(), |one| {
                let mut new = one.unwrap_or((remote_denom, Uint128::zero(), Uint128::zero()));
                new.1 += sent_amount;
                new.2 += sent_real_amount;
                StdResult::Ok(new)
            })?;
            if shares_amount == sent_amount {
                PENDING_LSM_SHARES.remove(deps.storage, denom.to_string());
            } else {
                PENDING_LSM_SHARES.update(deps.storage, denom.to_string(), |one| match one {
                    Some(one) => {
                        let mut new = one;
                        new.1 -= sent_amount;
                        new.2 -= sent_real_amount;
                        StdResult::Ok(new)
                    }
                    None => unreachable!("denom should be in the map"),
                })?;
            }
        }
    }

    deps.api.debug(&format!(
        "WASMDEBUG: transaction: {transaction:?}",
        transaction = transaction
    ));
    TX_STATE.save(deps.storage, &TxState::default())?;

    let config = CONFIG.load(deps.storage)?;

    deps.api.debug(&format!(
        "WASMDEBUG: json: {request:?}",
        request = to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
            ResponseHookMsg::Success(ResponseHookSuccessMsg {
                request_id: seq_id,
                request: request.clone(),
                transaction: transaction.clone(),
                answers: vec![ResponseAnswer::IBCTransfer(MsgIBCTransfer {})],
                local_height: env.block.height,
                remote_height: remote_height.u64(),
            },)
        ))?
    ));
    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
            ResponseHookMsg::Success(ResponseHookSuccessMsg {
                request_id: seq_id,
                request: request.clone(),
                transaction: transaction.clone(),
                answers: vec![ResponseAnswer::IBCTransfer(MsgIBCTransfer {})],
                local_height: env.block.height,
                remote_height: remote_height.u64(),
            }),
        ))?,
        funds: vec![],
    });

    Ok(response("sudo-response", "puppeteer", attrs).add_message(hook_message))
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

pub mod check_denom {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{QueryRequest, StdError, StdResult};

    use super::*;

    #[cw_serde]
    pub struct DenomData {
        pub remote_denom: String,
        pub validator: String,
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
    ) -> ContractResult<DenomData> {
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

        Ok(DenomData {
            remote_denom: trace.base_denom.to_string(),
            validator: validator.to_string(),
        })
    }
}
