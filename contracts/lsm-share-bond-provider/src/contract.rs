use cosmwasm_std::{
    attr, ensure_eq, to_json_binary, Addr, Attribute, Coin, CosmosMsg, CustomQuery, Decimal,
    Decimal256, Deps, StdResult, Uint128, Uint256, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_puppeteer_base::msg::IBCTransferReason;
use drop_puppeteer_base::state::RedeemShareItem;
use drop_staking_base::error::lsm_share_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::lsm_share_bond_provider::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::lsm_share_bond_provider::{
    Config, ConfigOptional, CONFIG, LAST_LSM_REDEEM, LSM_SHARES_TO_REDEEM, PENDING_LSM_SHARES,
    TOTAL_LSM_SHARES,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::interchain_queries::v047::types::DECIMAL_FRACTIONAL;
use prost::Message;

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
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let puppeteer_contract = deps.api.addr_validate(&msg.puppeteer_contract)?;
    let core_contract = deps.api.addr_validate(&msg.core_contract)?;
    let validators_set_contract = deps.api.addr_validate(&msg.validators_set_contract)?;
    let config = &Config {
        puppeteer_contract: puppeteer_contract.clone(),
        core_contract: core_contract.clone(),
        validators_set_contract: validators_set_contract.clone(),
        transfer_channel_id: msg.transfer_channel_id.to_string(),
        lsm_redeem_threshold: msg.lsm_redeem_threshold,
        lsm_redeem_maximum_interval: msg.lsm_redeem_maximum_interval,
    };
    CONFIG.save(deps.storage, config)?;

    TOTAL_LSM_SHARES.save(deps.storage, &0)?;
    LAST_LSM_REDEEM.save(deps.storage, &env.block.time.seconds())?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("puppeteer_contract", puppeteer_contract),
            attr("core_contract", core_contract),
            attr("validators_set_contract", validators_set_contract),
            attr("transfer_channel_id", msg.transfer_channel_id),
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
        QueryMsg::TokenAmount {
            coin,
            exchange_rate,
        } => query_token_amount(deps, coin, exchange_rate),
        QueryMsg::PendingLSMShares {} => query_pending_lsm_shares(deps),
        QueryMsg::LSMSharesToRedeem {} => query_lsm_shares_to_redeem(deps),
        QueryMsg::TotalLSMShares {} => {
            to_json_binary(&TOTAL_LSM_SHARES.load(deps.storage)?).map_err(From::from)
        }
    }
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
        || ((lsm_shares_to_redeem_count < lsm_redeem_threshold)
            && (last_lsm_redeem + config.lsm_redeem_maximum_interval < env.block.time.seconds()))
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
        ExecuteMsg::ProcessOnIdle {} => execute_process_on_idle(deps, env, info),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
    }
}

fn execute_process_on_idle(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let mut attrs = vec![attr("action", "process_on_idle")];
    let mut messages: Vec<CosmosMsg<NeutronMsg>> = vec![];

    attrs.push(attr("knot", "036"));
    if let Some(lsm_msg) = get_pending_redeem_msg(deps.as_ref(), &config, &env, info.funds.clone())?
    {
        messages.push(lsm_msg);
        attrs.push(attr("knot", "037"));
        attrs.push(attr("knot", "038"));
    } else {
        attrs.push(attr("knot", "041"));
        if let Some(lsm_msg) =
            get_pending_lsm_share_msg(deps.as_ref(), &config, &env, info.funds.clone())?
        {
            messages.push(lsm_msg);
            attrs.push(attr("knot", "042"));
            attrs.push(attr("knot", "043"));
        }
    }

    Ok(response("update_config", CONTRACT_NAME, Vec::<Attribute>::new()).add_attributes(attrs))
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

    if let Some(transfer_channel_id) = new_config.transfer_channel_id {
        state.transfer_channel_id = transfer_channel_id.to_string();
        attrs.push(attr("transfer_channel_id", transfer_channel_id))
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
    msg: drop_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );
    if let drop_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) = msg.clone() {
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
                    if let Some((remote_denom, shares_amount, real_amount)) = current_pending {
                        let sent_amount = Uint128::from(*amount);
                        LSM_SHARES_TO_REDEEM.update(deps.storage, denom.to_string(), |one| {
                            let mut new =
                                one.unwrap_or((remote_denom, Uint128::zero(), Uint128::zero()));
                            new.1 += sent_amount;
                            new.2 += real_amount;
                            StdResult::Ok(new)
                        })?;
                        if shares_amount == sent_amount {
                            PENDING_LSM_SHARES.remove(deps.storage, denom.to_string());
                        } else {
                            PENDING_LSM_SHARES.update(deps.storage, denom.to_string(), |one| {
                                match one {
                                    Some(one) => {
                                        let mut new = one;
                                        new.1 -= Uint128::from(*amount);
                                        new.2 -= real_amount;
                                        StdResult::Ok(new)
                                    }
                                    None => unreachable!("denom should be in the map"),
                                }
                            })?;
                        }
                    }
                }
            }
            drop_puppeteer_base::msg::Transaction::RedeemShares { items, .. } => {
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
            _ => {}
        }
    }

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    ))
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

pub fn get_pending_redeem_msg<T>(
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
            && (last_lsm_redeem + config.lsm_redeem_maximum_interval > env.block.time.seconds()))
    {
        return Ok(None);
    }
    let shares_to_redeeem = LSM_SHARES_TO_REDEEM
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .take(lsm_redeem_threshold)
        .collect::<StdResult<Vec<_>>>()?;

    let items = shares_to_redeeem
        .iter()
        .map(
            |(local_denom, (denom, share_amount, _real_amount))| RedeemShareItem {
                amount: *share_amount,
                local_denom: local_denom.to_string(),
                remote_denom: denom.to_string(),
            },
        )
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
    deps: Deps<X>,
    config: &Config,
    env: &Env,
    funds: Vec<cosmwasm_std::Coin>,
) -> ContractResult<Option<CosmosMsg<T>>> {
    let lsm_share: Option<(String, (String, Uint128, Uint128))> =
        PENDING_LSM_SHARES.first(deps.storage)?;
    match lsm_share {
        Some((local_denom, (_remote_denom, share_amount, _real_amount))) => {
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
                        amount: share_amount,
                    }];
                    all_funds.extend(funds);
                    all_funds
                },
            })))
        }
        None => Ok(None),
    }
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
