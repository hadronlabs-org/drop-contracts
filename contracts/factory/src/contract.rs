use std::collections::HashMap;

use cosmwasm_std::{
    attr, from_json, instantiate2_address, to_json_binary, Addr, Binary, CodeInfoResponse,
    CosmosMsg, Deps, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::ContractVersion;
use drop_helpers::answer::response;
use drop_helpers::phonebook::{
    CORE_CONTRACT, DISTRIBUTION_CONTRACT, LSM_SHARE_BOND_PROVIDER_CONTRACT,
    NATIVE_BOND_PROVIDER_CONTRACT, PUPPETEER_CONTRACT, REWARDS_MANAGER_CONTRACT,
    REWARDS_PUMP_CONTRACT, SPLITTER_CONTRACT, STRATEGY_CONTRACT, TOKEN_CONTRACT,
    UNBONDING_PUMP_CONTRACT, VALIDATORS_SET_CONTRACT, WITHDRAWAL_MANAGER_CONTRACT,
    WITHDRAWAL_VOUCHER_CONTRACT,
};
use drop_staking_base::error::factory::{ContractError, ContractResult};
use drop_staking_base::msg::factory::{
    ExecuteMsg, MigrateMsg, OwnerQueryMsg, ProxyMsg, QueryMsg, UpdateConfigMsg, ValidatorSetMsg,
};
use drop_staking_base::msg::{
    core::InstantiateMsg as CoreInstantiateMsg,
    distribution::InstantiateMsg as DistributionInstantiateMsg, factory::InstantiateMsg,
    rewards_manager::InstantiateMsg as RewardsMangerInstantiateMsg,
    splitter::InstantiateMsg as SplitterInstantiateMsg,
    strategy::InstantiateMsg as StrategyInstantiateMsg,
    token::InstantiateMsg as TokenInstantiateMsg,
    validatorset::InstantiateMsg as ValidatorsSetInstantiateMsg,
    withdrawal_manager::InstantiateMsg as WithdrawalManagerInstantiateMsg,
    withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
};
use drop_staking_base::state::factory::{CodeIds, PreInstantiatedContracts, STATE};
use drop_staking_base::state::splitter::Config as SplitterConfig;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const PERCENT_PRECISION: Uint128 = Uint128::new(10000u128); // allows to achieve 0.01% precision

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

    validate_pre_instantiated_contracts(
        deps.as_ref(),
        &env,
        &msg.pre_instantiated_contracts,
        &msg.code_ids,
    )?;

    let attrs = vec![
        attr("base_denom", &msg.base_denom),
        attr("salt", &msg.salt),
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
    let rewards_manager_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.rewards_manager_code_id)?;
    let splitter_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.splitter_code_id)?;
    let salt = msg.salt.as_bytes();

    let token_address =
        instantiate2_address(&token_contract_checksum, &canonical_self_address, salt)?;
    let core_address =
        instantiate2_address(&core_contract_checksum, &canonical_self_address, salt)?;
    let withdrawal_voucher_address = instantiate2_address(
        &withdrawal_voucher_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    let withdrawal_manager_address = instantiate2_address(
        &withdrawal_manager_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    let strategy_address =
        instantiate2_address(&strategy_contract_checksum, &canonical_self_address, salt)?;
    let validators_set_address = instantiate2_address(
        &validators_set_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    let distribution_calculator_address = instantiate2_address(
        &distribution_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    let rewards_manager_address = instantiate2_address(
        &rewards_manager_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    let splitter_address =
        instantiate2_address(&splitter_contract_checksum, &canonical_self_address, salt)?;

    let core_contract = deps.api.addr_humanize(&core_address)?;
    let token_contract = deps.api.addr_humanize(&token_address)?;
    let withdrawal_voucher_contract = deps.api.addr_humanize(&withdrawal_voucher_address)?;
    let withdrawal_manager_contract = deps.api.addr_humanize(&withdrawal_manager_address)?;
    let strategy_contract = deps.api.addr_humanize(&strategy_address)?;
    let validators_set_contract = deps.api.addr_humanize(&validators_set_address)?;
    let distribution_contract = deps.api.addr_humanize(&distribution_calculator_address)?;
    let rewards_manager_contract = deps.api.addr_humanize(&rewards_manager_address)?;
    let splitter_contract = deps.api.addr_humanize(&splitter_address)?;

    STATE.save(deps.storage, CORE_CONTRACT, &core_contract.clone())?;
    STATE.save(
        deps.storage,
        WITHDRAWAL_MANAGER_CONTRACT,
        &withdrawal_manager_contract.clone(),
    )?;
    STATE.save(
        deps.storage,
        REWARDS_MANAGER_CONTRACT,
        &rewards_manager_contract.clone(),
    )?;
    STATE.save(deps.storage, TOKEN_CONTRACT, &token_contract.clone())?;
    STATE.save(
        deps.storage,
        PUPPETEER_CONTRACT,
        &msg.pre_instantiated_contracts.puppeteer_address,
    )?;
    STATE.save(
        deps.storage,
        WITHDRAWAL_VOUCHER_CONTRACT,
        &withdrawal_voucher_contract.clone(),
    )?;
    STATE.save(deps.storage, STRATEGY_CONTRACT, &strategy_contract.clone())?;
    STATE.save(
        deps.storage,
        VALIDATORS_SET_CONTRACT,
        &validators_set_contract.clone(),
    )?;
    STATE.save(
        deps.storage,
        DISTRIBUTION_CONTRACT,
        &distribution_contract.clone(),
    )?;
    STATE.save(deps.storage, SPLITTER_CONTRACT, &splitter_contract.clone())?;
    if let Some(lsm_share_bond_provider_address) = &msg
        .pre_instantiated_contracts
        .lsm_share_bond_provider_address
        .clone()
    {
        STATE.save(
            deps.storage,
            LSM_SHARE_BOND_PROVIDER_CONTRACT,
            lsm_share_bond_provider_address,
        )?;
    }

    STATE.save(
        deps.storage,
        NATIVE_BOND_PROVIDER_CONTRACT,
        &msg.pre_instantiated_contracts
            .native_bond_provider_address
            .clone(),
    )?;
    if let Some(rewards_pump_address) = &msg.pre_instantiated_contracts.rewards_pump_address.clone()
    {
        STATE.save(deps.storage, REWARDS_PUMP_CONTRACT, rewards_pump_address)?;
    }
    if let Some(undonding_pump_address) = &msg
        .pre_instantiated_contracts
        .unbonding_pump_address
        .clone()
    {
        STATE.save(
            deps.storage,
            UNBONDING_PUMP_CONTRACT,
            undonding_pump_address,
        )?;
    }

    let msgs = vec![
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.token_code_id,
            label: get_contract_label("token"),
            msg: to_json_binary(&TokenInstantiateMsg {
                factory_contract: env.contract.address.to_string(),
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
            label: get_contract_label("validators-set"),
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
            label: get_contract_label("distribution"),
            msg: to_json_binary(&DistributionInstantiateMsg {})?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.strategy_code_id,
            label: get_contract_label("strategy"),
            msg: to_json_binary(&StrategyInstantiateMsg {
                owner: env.contract.address.to_string(),
                denom: msg.remote_opts.denom.to_string(),
                factory_contract: env.contract.address.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.core_code_id,
            label: get_contract_label("core"),
            msg: to_json_binary(&CoreInstantiateMsg {
                factory_contract: env.contract.address.to_string(),
                base_denom: msg.base_denom.clone(),
                remote_denom: msg.remote_opts.denom.to_string(),
                pump_ica_address: None,
                unbonding_period: msg.core_params.unbonding_period,
                unbonding_safe_period: msg.core_params.unbonding_safe_period,
                unbond_batch_switch_time: msg.core_params.unbond_batch_switch_time,
                idle_min_interval: msg.core_params.idle_min_interval,
                owner: env.contract.address.to_string(),
                emergency_address: None,
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
                symbol: "DROPV".to_string(),
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
                factory_contract: env.contract.address.to_string(),
                owner: env.contract.address.to_string(),
                base_denom: msg.base_denom.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.rewards_manager_code_id,
            label: get_contract_label("rewards-manager"),
            msg: to_json_binary(&RewardsMangerInstantiateMsg {
                owner: env.contract.address.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.splitter_code_id,
            label: get_contract_label("splitter"),
            msg: to_json_binary(&SplitterInstantiateMsg {
                config: SplitterConfig {
                    receivers: get_splitter_receivers(
                        msg.fee_params,
                        msg.pre_instantiated_contracts
                            .native_bond_provider_address
                            .to_string(),
                    )?,
                    denom: msg.base_denom.to_string(),
                },
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
    ];

    Ok(response("instantiate", CONTRACT_NAME, attrs).add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::State {} => query_state(deps),
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            Ok(to_json_binary(&ownership)?)
        }
    }
}

fn query_state(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let state = STATE.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);
    let out = state
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(k, v)| (k, v.into_string()))
        .collect::<HashMap<String, String>>();
    Ok(to_json_binary(&out)?)
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
    }
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
    let core_contract = STATE.load(deps.storage, CORE_CONTRACT)?;
    let validators_set_contract = STATE.load(deps.storage, VALIDATORS_SET_CONTRACT)?;
    let mut messages = vec![];
    match msg {
        UpdateConfigMsg::Core(msg) => messages.push(get_proxied_message(
            core_contract.to_string(),
            drop_staking_base::msg::core::ExecuteMsg::UpdateConfig {
                new_config: Box::new(*msg),
            },
            info.funds,
        )?),
        UpdateConfigMsg::ValidatorsSet(new_config) => messages.push(get_proxied_message(
            validators_set_contract.to_string(),
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
    let validators_set_contract = STATE.load(deps.storage, VALIDATORS_SET_CONTRACT)?;
    let puppeteer_contract = STATE.load(deps.storage, PUPPETEER_CONTRACT)?;
    let mut messages = vec![];
    let attrs = vec![attr("action", "proxy-call")];
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    match msg {
        ProxyMsg::ValidatorSet(msg) => match msg {
            ValidatorSetMsg::UpdateValidators { validators } => {
                messages.push(get_proxied_message(
                    validators_set_contract.to_string(),
                    drop_staking_base::msg::validatorset::ExecuteMsg::UpdateValidators {
                        validators: validators.clone(),
                    },
                    vec![],
                )?);
                messages.push(get_proxied_message(
                    puppeteer_contract.to_string(),
                    drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
                        validators: validators.iter().map(|v| { v.valoper_address.to_string() }).collect()
                    },
                    info.funds,
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
    let contract_version_metadata = cw2::get_contract_version(deps.storage)?;
    let storage_contract_name = contract_version_metadata.contract.as_str();
    if storage_contract_name != CONTRACT_NAME {
        return Err(ContractError::MigrationError {
            storage_contract_name: storage_contract_name.to_string(),
            contract_name: CONTRACT_NAME.to_string(),
        });
    }

    let storage_version: semver::Version = contract_version_metadata.version.parse()?;
    let version: semver::Version = CONTRACT_VERSION.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}

fn get_splitter_receivers(
    fee_params: Option<drop_staking_base::msg::factory::FeeParams>,
    bond_provider_address: String,
) -> ContractResult<Vec<(String, cosmwasm_std::Uint128)>> {
    match fee_params {
        Some(fee_params) => {
            let fee_weight = PERCENT_PRECISION * fee_params.fee;
            let bond_provider_weight = PERCENT_PRECISION - fee_weight;
            Ok(vec![
                (bond_provider_address, bond_provider_weight),
                (fee_params.fee_address, fee_weight),
            ])
        }
        None => Ok(vec![(bond_provider_address, PERCENT_PRECISION)]),
    }
}

pub fn get_contract_version(deps: Deps, contract_addr: &Addr) -> ContractResult<ContractVersion> {
    let contract_version = deps
        .querier
        .query_wasm_raw(contract_addr, b"contract_info")?;

    if let Some(contract_version) = contract_version {
        return Ok(from_json(&contract_version)?);
    }

    Err(ContractError::AbsentContractVersion {})
}

pub fn get_contract_config_owner(deps: Deps, contract_addr: &Addr) -> ContractResult<String> {
    let contract_owner: cw_ownable::Ownership<String> = deps
        .querier
        .query_wasm_smart(contract_addr, &OwnerQueryMsg::Ownership {})?;

    Ok(contract_owner.owner.unwrap_or_default())
}

pub fn validate_contract_metadata(
    deps: Deps,
    env: &Env,
    contract_addr: &Addr,
    valid_names: &[&str],
    code_id: u64,
) -> ContractResult<()> {
    let contract_version = get_contract_version(deps, contract_addr)?;

    if !valid_names.contains(&contract_version.contract.as_ref()) {
        return Err(ContractError::InvalidContractName {
            expected: valid_names.join(";"),
            actual: contract_version.contract,
        });
    }

    let contract_config_owner = get_contract_config_owner(deps, contract_addr)?;
    if contract_config_owner != env.contract.address {
        return Err(ContractError::InvalidContractOwner {
            contract: contract_addr.to_string(),
            expected: env.contract.address.to_string(),
            actual: contract_config_owner,
        });
    }

    let contract_info = deps.querier.query_wasm_contract_info(contract_addr)?;

    if let Some(contract_admin) = contract_info.admin {
        if contract_admin != env.contract.address {
            return Err(ContractError::InvalidContractAdmin {
                contract: contract_addr.to_string(),
                expected: env.contract.address.to_string(),
                actual: contract_admin.to_string(),
            });
        }
    } else {
        return Err(ContractError::InvalidContractAdmin {
            contract: contract_addr.to_string(),
            expected: env.contract.address.to_string(),
            actual: "None".to_string(),
        });
    }

    if contract_info.code_id != code_id {
        return Err(ContractError::InvalidContractCodeId {
            contract: contract_addr.to_string(),
            expected: code_id.to_string(),
            actual: contract_info.code_id.to_string(),
        });
    }

    Ok(())
}

fn validate_pre_instantiated_contracts(
    deps: Deps,
    env: &Env,
    pre_instantiated_contracts: &PreInstantiatedContracts,
    code_ids: &CodeIds,
) -> Result<(), ContractError> {
    // Validate native bond provider contract
    validate_contract_metadata(
        deps,
        env,
        &pre_instantiated_contracts.native_bond_provider_address,
        &[
            drop_native_bond_provider::contract::CONTRACT_NAME,
            drop_native_sync_bond_provider::contract::CONTRACT_NAME,
        ],
        code_ids.native_bond_provider_code_id.unwrap_or_default(),
    )?;

    // Validate val ref address
    if let Some(val_ref_address) = &pre_instantiated_contracts.val_ref_address {
        validate_contract_metadata(
            deps,
            env,
            val_ref_address,
            &[drop_val_ref::contract::CONTRACT_NAME],
            code_ids.val_ref_code_id.unwrap_or_default(),
        )?;
    }

    // Validate lsm share bond provider contract
    if let Some(lsm_share_bond_provider_address) =
        &pre_instantiated_contracts.lsm_share_bond_provider_address
    {
        validate_contract_metadata(
            deps,
            env,
            lsm_share_bond_provider_address,
            &[drop_lsm_share_bond_provider::contract::CONTRACT_NAME],
            code_ids.lsm_share_bond_provider_code_id.unwrap_or_default(),
        )?;
    }

    // Validate puppeteer contract
    validate_contract_metadata(
        deps,
        env,
        &pre_instantiated_contracts.puppeteer_address,
        &[
            drop_puppeteer::contract::CONTRACT_NAME,
            drop_puppeteer_initia::contract::CONTRACT_NAME,
            drop_puppeteer_native::contract::CONTRACT_NAME,
        ],
        code_ids.puppeteer_code_id.unwrap_or_default(),
    )?;

    // Validate unbonding and rewards pump contracts
    if let Some(unbonding_pump_address) = &pre_instantiated_contracts.unbonding_pump_address {
        validate_contract_metadata(
            deps,
            env,
            unbonding_pump_address,
            &[drop_pump::contract::CONTRACT_NAME],
            code_ids.unbonding_pump_code_id.unwrap_or_default(),
        )?;
    }
    if let Some(rewards_pump_address) = &pre_instantiated_contracts.rewards_pump_address {
        validate_contract_metadata(
            deps,
            env,
            rewards_pump_address,
            &[drop_pump::contract::CONTRACT_NAME],
            code_ids.rewards_pump_code_id.unwrap_or_default(),
        )?;
    }

    Ok(())
}
