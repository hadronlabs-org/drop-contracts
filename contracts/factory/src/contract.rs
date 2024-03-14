use crate::{
    error::ContractResult,
    msg::{
        CallbackMsg, CoreParams, ExecuteMsg, InstantiateMsg, ProxyMsg, QueryMsg, UpdateConfigMsg,
        ValidatorSetMsg,
    },
    state::{Config, State, CONFIG, STATE},
};
use cosmwasm_std::{
    attr, entry_point, instantiate2_address, to_json_binary, Binary, CodeInfoResponse, CosmosMsg,
    Deps, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use lido_helpers::answer::response;
use lido_staking_base::{
    msg::core::{ExecuteMsg as CoreExecuteMsg, InstantiateMsg as CoreInstantiateMsg},
    msg::distribution::InstantiateMsg as DistributionInstantiateMsg,
    msg::puppeteer::InstantiateMsg as PuppeteerInstantiateMsg,
    msg::rewards_manager::InstantiateMsg as RewardsMangerInstantiateMsg,
    msg::strategy::InstantiateMsg as StrategyInstantiateMsg,
    msg::token::{
        ConfigResponse as TokenConfigResponse, InstantiateMsg as TokenInstantiateMsg,
        QueryMsg as TokenQueryMsg,
    },
    msg::validatorset::InstantiateMsg as ValidatorsSetInstantiateMsg,
    msg::withdrawal_manager::InstantiateMsg as WithdrawalManagerInstantiateMsg,
    msg::withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let attrs = vec![
        attr("salt", &msg.salt),
        attr("code_ids", format!("{:?}", &msg.code_ids)),
        attr("remote_opts", format!("{:?}", &msg.remote_opts)),
        attr("owner", &info.sender),
        attr("subdenom", &msg.subdenom),
    ];
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    CONFIG.save(
        deps.storage,
        &Config {
            salt: msg.salt.to_string(),
            code_ids: msg.code_ids,
            remote_opts: msg.remote_opts,
            subdenom: msg.subdenom.to_string(),
            sdk_version: msg.sdk_version,
            token_metadata: msg.token_metadata,
        },
    )?;

    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Init {
            base_denom,
            core_params,
        } => execute_init(deps, env, info, base_denom, core_params),
        ExecuteMsg::Callback(msg) => match msg {
            CallbackMsg::PostInit {} => execute_post_init(deps, env, info),
        },
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
        ExecuteMsg::AdminExecute { addr, msg } => execute_admin_execute(deps, env, info, addr, msg),
    }
}

fn execute_admin_execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addr: String,
    msg: Binary,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "admin-execute")];
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addr,
        msg,
        funds: vec![],
    });
    Ok(response("execute-admin", CONTRACT_NAME, attrs).add_message(msg))
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
            lido_staking_base::msg::core::ExecuteMsg::UpdateConfig {
                new_config: Box::new(*msg),
            },
            info.funds,
        )?),
        UpdateConfigMsg::ValidatorsSet(new_config) => messages.push(get_proxied_message(
            state.validators_set_contract,
            lido_staking_base::msg::validatorset::ExecuteMsg::UpdateConfig { new_config },
            info.funds,
        )?),
        UpdateConfigMsg::PuppeteerFees(fees) => messages.push(get_proxied_message(
            state.puppeteer_contract,
            lido_puppeteer_base::msg::ExecuteMsg::SetFees {
                recv_fee: fees.recv_fee,
                ack_fee: fees.ack_fee,
                timeout_fee: fees.timeout_fee,
                register_fee: fees.register_fee,
            },
            info.funds,
        )?),
    }
    Ok(response("execute-proxy-call", CONTRACT_NAME, attrs).add_messages(messages))
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
                    lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
                        validators: validators.clone(),
                    },
                    vec![],
                )?);
                messages.push(get_proxied_message(
                    state.puppeteer_contract,
                    lido_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery { validators: validators.iter().map(|v| {v.valoper_address.to_string()}).collect() },
                    info.funds,
                )?)
            }
            ValidatorSetMsg::UpdateValidator { validator } => messages.push(get_proxied_message(
                state.validators_set_contract,
                lido_staking_base::msg::validatorset::ExecuteMsg::UpdateValidator { validator },
                info.funds,
            )?),
        },
        ProxyMsg::Core(msg) => match msg {
            crate::msg::CoreMsg::UpdateNonNativeRewardsReceivers { items } => {
                messages.push(get_proxied_message(
                    state.core_contract,
                    lido_staking_base::msg::core::ExecuteMsg::UpdateNonNativeRewardsReceivers {
                        items: items.clone(),
                    },
                    vec![],
                )?);
                messages.push(
                    get_proxied_message(
                        state.puppeteer_contract,
                        lido_staking_base::msg::puppeteer::ExecuteMsg::RegisterNonNativeRewardsBalancesQuery {
                            denoms: items.iter().map(|one|{one.denom.to_string()}).collect() }, info.funds)?
                );
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

fn execute_init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    base_denom: String,
    core_params: CoreParams,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let canonical_self_address = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let mut attrs = vec![
        attr("action", "init"),
        attr("base_denom", &base_denom),
        attr("sdk_version", config.sdk_version),
    ];

    let token_contract_checksum = get_code_checksum(deps.as_ref(), config.code_ids.token_code_id)?;
    let core_contract_checksum = get_code_checksum(deps.as_ref(), config.code_ids.core_code_id)?;
    let withdrawal_voucher_contract_checksum =
        get_code_checksum(deps.as_ref(), config.code_ids.withdrawal_voucher_code_id)?;
    let withdrawal_manager_contract_checksum =
        get_code_checksum(deps.as_ref(), config.code_ids.withdrawal_manager_code_id)?;
    let strategy_contract_checksum =
        get_code_checksum(deps.as_ref(), config.code_ids.strategy_code_id)?;
    let validators_set_contract_checksum =
        get_code_checksum(deps.as_ref(), config.code_ids.validators_set_code_id)?;
    let distribution_contract_checksum =
        get_code_checksum(deps.as_ref(), config.code_ids.distribution_code_id)?;
    let puppeteer_contract_checksum =
        get_code_checksum(deps.as_ref(), config.code_ids.puppeteer_code_id)?;
    let rewards_manager_contract_checksum =
        get_code_checksum(deps.as_ref(), config.code_ids.rewards_manager_code_id)?;

    let salt = config.salt.as_bytes();

    let token_address =
        instantiate2_address(&token_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("token_address", token_address.to_string()));
    let core_address =
        instantiate2_address(&core_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("core_address", core_address.to_string()));
    let puppeteer_address =
        instantiate2_address(&puppeteer_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("core_address", core_address.to_string()));

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
    let rewards_manager_contract = deps
        .api
        .addr_humanize(&rewards_manager_address)?
        .to_string();

    let state = State {
        token_contract: token_contract.to_string(),
        core_contract: core_contract.to_string(),
        puppeteer_contract: puppeteer_contract.to_string(),
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
            code_id: config.code_ids.token_code_id,
            label: get_contract_label("token"),
            msg: to_json_binary(&TokenInstantiateMsg {
                core_address: core_contract.to_string(),
                subdenom: config.subdenom,
                token_metadata: config.token_metadata,
                owner: env.contract.address.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.code_ids.validators_set_code_id,
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
            code_id: config.code_ids.distribution_code_id,
            label: "distribution".to_string(),
            msg: to_json_binary(&DistributionInstantiateMsg {})?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.code_ids.puppeteer_code_id,
            label: get_contract_label("puppeteer"),
            msg: to_json_binary(&PuppeteerInstantiateMsg {
                allowed_senders: vec![core_contract.to_string()],
                owner: env.contract.address.to_string(),
                remote_denom: config.remote_opts.denom.to_string(),
                update_period: config.remote_opts.update_period,
                connection_id: config.remote_opts.connection_id.to_string(),
                port_id: config.remote_opts.port_id.to_string(),
                transfer_channel_id: config.remote_opts.transfer_channel_id.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.code_ids.strategy_code_id,
            label: "strategy".to_string(),
            msg: to_json_binary(&StrategyInstantiateMsg {
                core_address: env.contract.address.to_string(),
                puppeteer_address: puppeteer_contract.to_string(),
                validator_set_address: validators_set_contract.to_string(),
                distribution_address: distribution_contract.to_string(),
                denom: config.remote_opts.denom.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.code_ids.core_code_id,
            label: get_contract_label("core"),
            msg: to_json_binary(&CoreInstantiateMsg {
                token_contract: token_contract.to_string(),
                puppeteer_contract: puppeteer_contract.to_string(),
                strategy_contract: strategy_contract.to_string(),
                withdrawal_voucher_contract: withdrawal_voucher_contract.to_string(),
                withdrawal_manager_contract: withdrawal_manager_contract.to_string(),
                base_denom: base_denom.to_string(),
                remote_denom: config.remote_opts.denom.to_string(),
                pump_address: None,
                validators_set_contract,
                puppeteer_timeout: core_params.puppeteer_timeout,
                unbonding_period: core_params.unbonding_period,
                unbonding_safe_period: core_params.unbonding_safe_period,
                unbond_batch_switch_time: core_params.unbond_batch_switch_time,
                idle_min_interval: core_params.idle_min_interval,
                bond_limit: core_params.bond_limit,
                channel: core_params.channel,
                lsm_redeem_threshold: core_params.lsm_redeem_threshold,
                owner: env.contract.address.to_string(),
                fee: None,
                fee_address: None,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.code_ids.withdrawal_voucher_code_id,
            label: get_contract_label("withdrawal-voucher"),
            msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
                name: "Lido Voucher".to_string(),
                symbol: "LDOV".to_string(),
                minter: core_contract.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.code_ids.withdrawal_manager_code_id,
            label: get_contract_label("withdrawal-manager"),
            msg: to_json_binary(&WithdrawalManagerInstantiateMsg {
                core_contract: core_contract.to_string(),
                voucher_contract: withdrawal_voucher_contract.to_string(),
                owner: env.contract.address.to_string(),
                base_denom,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.code_ids.rewards_manager_code_id,
            label: get_contract_label("rewards manager"),
            msg: to_json_binary(&RewardsMangerInstantiateMsg {
                core_address: core_contract.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&ExecuteMsg::Callback(CallbackMsg::PostInit {}))?,
            funds: vec![],
        }),
    ];

    Ok(response("execute-init", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn execute_post_init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "post_init")];
    let state = STATE.load(deps.storage)?;
    let token_config: TokenConfigResponse = deps
        .querier
        .query_wasm_smart(state.token_contract, &TokenQueryMsg::Config {})?;
    let core_update_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.core_contract,
        msg: to_json_binary(&CoreExecuteMsg::UpdateConfig {
            new_config: Box::new(lido_staking_base::state::core::ConfigOptional {
                ld_denom: Some(token_config.denom),
                ..lido_staking_base::state::core::ConfigOptional::default()
            }),
        })?,
        funds: vec![],
    });
    Ok(response("execute-post_init", CONTRACT_NAME, attrs).add_message(core_update_msg))
}

fn get_code_checksum(deps: Deps, code_id: u64) -> NeutronResult<HexBinary> {
    let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;
    Ok(checksum)
}

fn get_contract_label(base: &str) -> String {
    format!("LIDO-staking-{}", base)
}
