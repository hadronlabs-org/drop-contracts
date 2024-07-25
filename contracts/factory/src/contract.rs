use crate::{
    error::ContractResult,
    msg::{
        ExecuteMsg, InstantiateMsg, MigrateMsg, ProxyMsg, QueryMsg, UpdateConfigMsg,
        ValidatorSetMsg,
    },
    state::{State, STATE},
};
use cosmwasm_std::{
    attr, instantiate2_address, to_json_binary, Attribute, Binary, CodeInfoResponse, CosmosMsg,
    Deps, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, WasmMsg,
};
use drop_helpers::answer::response;
use drop_staking_base::msg::{
    core::{InstantiateMsg as CoreInstantiateMsg, QueryMsg as CoreQueryMsg},
    distribution::InstantiateMsg as DistributionInstantiateMsg,
    puppeteer::InstantiateMsg as PuppeteerInstantiateMsg,
    rewards_manager::{InstantiateMsg as RewardsMangerInstantiateMsg, QueryMsg as RewardsQueryMsg},
    staker::InstantiateMsg as StakerInstantiateMsg,
    strategy::InstantiateMsg as StrategyInstantiateMsg,
    token::InstantiateMsg as TokenInstantiateMsg,
    validatorset::InstantiateMsg as ValidatorsSetInstantiateMsg,
    withdrawal_manager::{
        InstantiateMsg as WithdrawalManagerInstantiateMsg, QueryMsg as WithdrawalManagerQueryMsg,
    },
    withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    let mut attrs = vec![
        attr("action", "init"),
        attr("base_denom", &msg.base_denom),
        attr("sdk_version", &msg.sdk_version),
        attr("salt", &msg.salt),
        attr("code_ids", format!("{:?}", msg.code_ids)),
        attr("remote_opts", format!("{:?}", msg.remote_opts)),
        attr("owner", info.sender),
        attr("subdenom", &msg.subdenom),
    ];

    let canonical_self_address = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let token_contract_checksum = get_code_checksum(deps.as_ref(), msg.code_ids.token_code_id)?;
    let core_contract_checksum = get_code_checksum(deps.as_ref(), msg.code_ids.core_code_id)?;
    let withdrawal_voucher_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.withdrawal_voucher_code_id)?;
    let withdrawal_manager_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.withdrawal_manager_code_id)?;
    let strategy_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.strategy_code_id)?;
    let validators_set_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.validators_set_code_id)?;
    let distribution_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.distribution_code_id)?;
    let puppeteer_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.puppeteer_code_id)?;
    let staker_contract_checksum = get_code_checksum(deps.as_ref(), msg.code_ids.staker_code_id)?;
    let rewards_manager_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.rewards_manager_code_id)?;
    let salt = msg.salt.as_bytes();

    let token_address =
        instantiate2_address(&token_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("token_address", token_address.to_string()));
    let core_address =
        instantiate2_address(&core_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("core_address", core_address.to_string()));
    let puppeteer_address =
        instantiate2_address(&puppeteer_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("puppeteer_address", puppeteer_address.to_string()));
    let staker_address =
        instantiate2_address(&staker_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("staker_address", staker_address.to_string()));

    let withdrawal_voucher_address = instantiate2_address(
        &withdrawal_voucher_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    attrs.push(attr(
        "withdrawal_voucher_address",
        withdrawal_voucher_address.to_string(),
    ));

    let withdrawal_manager_address = instantiate2_address(
        &withdrawal_manager_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    attrs.push(attr(
        "withdrawal_manager_address",
        withdrawal_manager_address.to_string(),
    ));

    let strategy_address =
        instantiate2_address(&strategy_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("strategy_address", strategy_address.to_string()));

    let validators_set_address = instantiate2_address(
        &validators_set_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    attrs.push(attr(
        "validators_set_address",
        validators_set_address.to_string(),
    ));

    let distribution_address = instantiate2_address(
        &distribution_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    attrs.push(attr(
        "distribution_address",
        distribution_address.to_string(),
    ));

    let rewards_manager_address = instantiate2_address(
        &rewards_manager_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    attrs.push(attr(
        "rewards_manager_address",
        rewards_manager_address.to_string(),
    ));

    let core_contract = deps.api.addr_humanize(&core_address)?.to_string();
    let token_contract = deps.api.addr_humanize(&token_address)?.to_string();
    let withdrawal_voucher_contract = deps
        .api
        .addr_humanize(&withdrawal_voucher_address)?
        .to_string();
    let withdrawal_manager_contract = deps
        .api
        .addr_humanize(&withdrawal_manager_address)?
        .to_string();
    let strategy_contract = deps.api.addr_humanize(&strategy_address)?.to_string();
    let validators_set_contract = deps.api.addr_humanize(&validators_set_address)?.to_string();
    let distribution_contract = deps.api.addr_humanize(&distribution_address)?.to_string();
    let puppeteer_contract = deps.api.addr_humanize(&puppeteer_address)?.to_string();
    let staker_contract = deps.api.addr_humanize(&staker_address)?.to_string();
    let rewards_manager_contract = deps
        .api
        .addr_humanize(&rewards_manager_address)?
        .to_string();

    let state = State {
        token_contract: token_contract.to_string(),
        core_contract: core_contract.to_string(),
        puppeteer_contract: puppeteer_contract.to_string(),
        staker_contract: staker_contract.to_string(),
        withdrawal_voucher_contract: withdrawal_voucher_contract.to_string(),
        withdrawal_manager_contract: withdrawal_manager_contract.to_string(),
        strategy_contract: strategy_contract.to_string(),
        validators_set_contract: validators_set_contract.to_string(),
        distribution_contract: distribution_contract.to_string(),
        rewards_manager_contract: rewards_manager_contract.to_string(),
    };
    STATE.save(deps.storage, &state)?;

    let msgs = vec![
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.token_code_id,
            label: get_contract_label("token"),
            msg: to_json_binary(&TokenInstantiateMsg {
                core_address: core_contract.to_string(),
                subdenom: msg.subdenom,
                token_metadata: msg.token_metadata,
                owner: env.contract.address.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.validators_set_code_id,
            label: "validators set".to_string(),
            msg: to_json_binary(&ValidatorsSetInstantiateMsg {
                stats_contract: "neutron1x69dz0c0emw8m2c6kp5v6c08kgjxmu30f4a8w5".to_string(), //FIXME: mock address, replace with real one
                owner: env.contract.address.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.distribution_code_id,
            label: "distribution".to_string(),
            msg: to_json_binary(&DistributionInstantiateMsg {})?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.puppeteer_code_id,
            label: get_contract_label("puppeteer"),
            msg: to_json_binary(&PuppeteerInstantiateMsg {
                allowed_senders: vec![core_contract.to_string(), env.contract.address.to_string()],
                owner: Some(env.contract.address.to_string()),
                remote_denom: msg.remote_opts.denom.to_string(),
                update_period: msg.remote_opts.update_period,
                connection_id: msg.remote_opts.connection_id.to_string(),
                port_id: msg.remote_opts.port_id.to_string(),
                transfer_channel_id: msg.remote_opts.transfer_channel_id.to_string(),
                sdk_version: msg.sdk_version.to_string(),
                timeout: msg.puppeteer_params.timeout,
                delegations_queries_chunk_size: None,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.staker_code_id,
            label: get_contract_label("staker"),
            msg: to_json_binary(&StakerInstantiateMsg {
                allowed_senders: vec![core_contract.to_string()],
                owner: Some(env.contract.address.to_string()),
                remote_denom: msg.remote_opts.denom.to_string(),
                connection_id: msg.remote_opts.connection_id.to_string(),
                port_id: msg.remote_opts.port_id.to_string(),
                transfer_channel_id: msg.remote_opts.transfer_channel_id.to_string(),
                timeout: msg.staker_params.timeout,
                base_denom: msg.base_denom.clone(),
                min_ibc_transfer: msg.staker_params.min_ibc_transfer,
                min_staking_amount: msg.staker_params.min_stake_amount,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.strategy_code_id,
            label: "strategy".to_string(),
            msg: to_json_binary(&StrategyInstantiateMsg {
                owner: env.contract.address.to_string(),
                puppeteer_address: puppeteer_contract.to_string(),
                validator_set_address: validators_set_contract.to_string(),
                distribution_address: distribution_contract.to_string(),
                denom: msg.remote_opts.denom.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.core_code_id,
            label: get_contract_label("core"),
            msg: to_json_binary(&CoreInstantiateMsg {
                token_contract: token_contract.to_string(),
                puppeteer_contract: puppeteer_contract.to_string(),
                staker_contract: staker_contract.to_string(),
                strategy_contract: strategy_contract.to_string(),
                withdrawal_voucher_contract: withdrawal_voucher_contract.to_string(),
                withdrawal_manager_contract: withdrawal_manager_contract.to_string(),
                base_denom: msg.base_denom.clone(),
                remote_denom: msg.remote_opts.denom.to_string(),
                pump_ica_address: None,
                validators_set_contract,
                unbonding_period: msg.core_params.unbonding_period,
                unbonding_safe_period: msg.core_params.unbonding_safe_period,
                unbond_batch_switch_time: msg.core_params.unbond_batch_switch_time,
                idle_min_interval: msg.core_params.idle_min_interval,
                bond_limit: msg.core_params.bond_limit,
                transfer_channel_id: msg.remote_opts.transfer_channel_id,
                lsm_min_bond_amount: msg.core_params.lsm_min_bond_amount,
                lsm_redeem_threshold: msg.core_params.lsm_redeem_threshold,
                lsm_redeem_max_interval: msg.core_params.lsm_redeem_max_interval,
                owner: env.contract.address.to_string(),
                fee: None,
                fee_address: None,
                emergency_address: None,
                min_stake_amount: msg.core_params.min_stake_amount,
                icq_update_delay: msg.core_params.icq_update_delay,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.withdrawal_voucher_code_id,
            label: get_contract_label("withdrawal-voucher"),
            msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
                name: "Drop Voucher".to_string(),
                symbol: "LDOV".to_string(),
                minter: core_contract.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.withdrawal_manager_code_id,
            label: get_contract_label("withdrawal-manager"),
            msg: to_json_binary(&WithdrawalManagerInstantiateMsg {
                core_contract: core_contract.to_string(),
                voucher_contract: withdrawal_voucher_contract.to_string(),
                owner: env.contract.address.to_string(),
                base_denom: msg.base_denom,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.rewards_manager_code_id,
            label: get_contract_label("rewards manager"),
            msg: to_json_binary(&RewardsMangerInstantiateMsg {
                owner: env.contract.address.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
    ];

    Ok(response("instantiate", CONTRACT_NAME, attrs).add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?),
        QueryMsg::PauseInfo {} => query_pause_info(deps),
    }
}

fn query_pause_info(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;

    to_json_binary(&crate::state::PauseInfoResponse {
        core: deps
            .querier
            .query_wasm_smart(state.core_contract, &CoreQueryMsg::PauseInfo {})?,
        withdrawal_manager: deps.querier.query_wasm_smart(
            state.withdrawal_manager_contract,
            &WithdrawalManagerQueryMsg::PauseInfo {},
        )?,
        rewards_manager: deps.querier.query_wasm_smart(
            state.rewards_manager_contract,
            &RewardsQueryMsg::PauseInfo {},
        )?,
    })
    .map_err(From::from)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::UpdateConfig(msg) => execute_update_config(deps, env, info, *msg),
        ExecuteMsg::Proxy(msg) => execute_proxy_msg(deps, env, info, msg),
        ExecuteMsg::AdminExecute { msgs } => execute_admin_execute(deps, env, info, msgs),
        ExecuteMsg::Pause {} => exec_pause(deps, info),
        ExecuteMsg::Unpause {} => exec_unpause(deps, info),
    }
}

fn exec_pause(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let state = STATE.load(deps.storage)?;

    let messages = vec![
        get_proxied_message(
            state.core_contract,
            drop_staking_base::msg::core::ExecuteMsg::Pause {},
            vec![],
        )?,
        get_proxied_message(
            state.withdrawal_manager_contract,
            drop_staking_base::msg::withdrawal_manager::ExecuteMsg::Pause {},
            vec![],
        )?,
        get_proxied_message(
            state.rewards_manager_contract,
            drop_staking_base::msg::rewards_manager::ExecuteMsg::Pause {},
            vec![],
        )?,
    ];

    Ok(response("execute-pause", CONTRACT_NAME, Vec::<Attribute>::new()).add_messages(messages))
}

fn exec_unpause(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let state = STATE.load(deps.storage)?;

    let messages = vec![
        get_proxied_message(
            state.core_contract,
            drop_staking_base::msg::core::ExecuteMsg::Unpause {},
            vec![],
        )?,
        get_proxied_message(
            state.rewards_manager_contract,
            drop_staking_base::msg::rewards_manager::ExecuteMsg::Unpause {},
            vec![],
        )?,
        get_proxied_message(
            state.withdrawal_manager_contract,
            drop_staking_base::msg::withdrawal_manager::ExecuteMsg::Unpause {},
            vec![],
        )?,
    ];

    Ok(response("execute-unpause", CONTRACT_NAME, Vec::<Attribute>::new()).add_messages(messages))
}

fn execute_admin_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<NeutronMsg>>,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "admin-execute")];
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    Ok(response("execute-admin", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "update-config")];
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let state = STATE.load(deps.storage)?;
    let mut messages = vec![];
    match msg {
        UpdateConfigMsg::Core(msg) => messages.push(get_proxied_message(
            state.core_contract,
            drop_staking_base::msg::core::ExecuteMsg::UpdateConfig {
                new_config: Box::new(*msg),
            },
            info.funds,
        )?),
        UpdateConfigMsg::ValidatorsSet(new_config) => messages.push(get_proxied_message(
            state.validators_set_contract,
            drop_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig { new_config },
            info.funds,
        )?),
    }
    Ok(response("execute-update-config", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_proxy_msg(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ProxyMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let state = STATE.load(deps.storage)?;
    let mut messages = vec![];
    let attrs = vec![attr("action", "proxy-call")];
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    match msg {
        ProxyMsg::ValidatorSet(msg) => match msg {
            ValidatorSetMsg::UpdateValidators { validators } => {
                messages.push(get_proxied_message(
                    state.validators_set_contract,
                    drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
                        validators: validators.clone(),
                    },
                    vec![],
                )?);
                messages.push(get_proxied_message(
                    state.puppeteer_contract,
                    drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery { validators: validators.iter().map(|v| {v.valoper_address.to_string()}).collect() },
                    info.funds,
                )?)
            }
        },
        ProxyMsg::Core(msg) => match msg {
            crate::msg::CoreMsg::UpdateNonNativeRewardsReceivers { items } => {
                messages.push(get_proxied_message(
                    state.core_contract,
                    drop_staking_base::msg::core::ExecuteMsg::UpdateNonNativeRewardsReceivers {
                        items: items.clone(),
                    },
                    vec![],
                )?);
                messages.push(
                    get_proxied_message(
                        state.puppeteer_contract,
                        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterNonNativeRewardsBalancesQuery {
                            denoms: items.iter().map(|one|{one.denom.to_string()}).collect() }, info.funds)?
                );
            }
            crate::msg::CoreMsg::Pause {} => {
                messages.push(get_proxied_message(
                    state.core_contract,
                    drop_staking_base::msg::core::ExecuteMsg::Pause {},
                    vec![],
                )?);
            }
            crate::msg::CoreMsg::Unpause {} => {
                messages.push(get_proxied_message(
                    state.core_contract,
                    drop_staking_base::msg::core::ExecuteMsg::Unpause {},
                    vec![],
                )?);
            }
        },
    }
    Ok(response("execute-proxy-call", CONTRACT_NAME, attrs).add_messages(messages))
}

fn get_proxied_message<T: cosmwasm_schema::serde::Serialize>(
    contract_addr: String,
    msg: T,
    funds: Vec<cosmwasm_std::Coin>,
) -> ContractResult<CosmosMsg<NeutronMsg>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_json_binary(&msg)?,
        funds,
    }))
}

fn get_code_checksum(deps: Deps, code_id: u64) -> NeutronResult<HexBinary> {
    let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;
    Ok(checksum)
}

fn get_contract_label(base: &str) -> String {
    format!("drop-staking-{}", base)
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
