use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, from_json, instantiate2_address, to_json_binary, Addr, Binary, CodeInfoResponse,
    CosmosMsg, Deps, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::ContractVersion;
use cw_storage_plus::Item;
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
    lsm_share_bond_provider::InstantiateMsg as LsmShareBondProviderInstantiateMsg,
    native_bond_provider::InstantiateMsg as NativeBondProviderInstantiateMsg,
    rewards_manager::InstantiateMsg as RewardsMangerInstantiateMsg,
    splitter::InstantiateMsg as SplitterInstantiateMsg,
    strategy::InstantiateMsg as StrategyInstantiateMsg,
    token::InstantiateMsg as TokenInstantiateMsg,
    validatorset::InstantiateMsg as ValidatorsSetInstantiateMsg,
    withdrawal_manager::InstantiateMsg as WithdrawalManagerInstantiateMsg,
    withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
};
use drop_staking_base::state::factory::{PreInstantiatedContracts, STATE};
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

    validate_pre_instantiated_contracts(deps.as_ref(), &env, &msg.pre_instantiated_contracts)?;

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
    env: Env,
    msg: MigrateMsg,
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

        #[cw_serde]
        pub struct OldState {
            pub token_contract: String,
            pub core_contract: String,
            pub puppeteer_contract: String,
            pub staker_contract: String,
            pub withdrawal_voucher_contract: String,
            pub withdrawal_manager_contract: String,
            pub strategy_contract: String,
            pub validators_set_contract: String,
            pub distribution_contract: String,
            pub rewards_manager_contract: String,
            pub rewards_pump_contract: String,
            pub splitter_contract: String,
        }

        let state = Item::<OldState>::new("state").load(deps.storage)?;

        let mut messages = vec![];

        let salt = msg.salt.as_bytes();

        let canonical_self_address = deps.api.addr_canonicalize(env.contract.address.as_str())?;

        let lsm_share_bond_provider_checksum = get_code_checksum(
            deps.as_ref().into_empty(),
            msg.lsm_share_bond_provider_code_id,
        )?;

        let lsm_share_bond_provider_address = instantiate2_address(
            &lsm_share_bond_provider_checksum,
            &canonical_self_address,
            salt,
        )?;

        let lsm_share_bond_provider_contract = deps
            .api
            .addr_humanize(&lsm_share_bond_provider_address)?
            .to_string();

        let core_config = deps
            .querier
            .query_wasm_smart::<drop_staking_base::state::core::OldConfig>(
                &state.core_contract,
                &drop_staking_base::msg::core::QueryMsg::Config {},
            )?;

        messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.lsm_share_bond_provider_code_id,
            label: get_contract_label("lsm_share_bond_provider"),
            msg: to_json_binary(&LsmShareBondProviderInstantiateMsg {
                owner: env.contract.address.to_string(),
                factory_contract: env.contract.address.to_string(),
                port_id: msg.port_id.to_string(),
                transfer_channel_id: core_config.transfer_channel_id.to_string(),
                timeout: msg.timeout,
                lsm_min_bond_amount: core_config.lsm_min_bond_amount,
                lsm_redeem_threshold: core_config.lsm_redeem_threshold,
                lsm_redeem_maximum_interval: core_config.lsm_redeem_maximum_interval,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }));

        let native_bond_provider_checksum =
            get_code_checksum(deps.as_ref().into_empty(), msg.native_bond_provider_code_id)?;

        let native_bond_provider_address = instantiate2_address(
            &native_bond_provider_checksum,
            &canonical_self_address,
            salt,
        )?;

        let native_bond_provider_contract = deps
            .api
            .addr_humanize(&native_bond_provider_address)?
            .to_string();

        messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.native_bond_provider_code_id,
            label: get_contract_label("native_bond_provider"),
            msg: to_json_binary(&NativeBondProviderInstantiateMsg {
                owner: env.contract.address.to_string(),
                base_denom: core_config.base_denom.to_string(),
                factory_contract: env.contract.address.to_string(),
                min_ibc_transfer: msg.min_ibc_transfer,
                min_stake_amount: core_config.min_stake_amount,
                transfer_channel_id: core_config.transfer_channel_id.to_string(),
                port_id: msg.port_id.to_string(),
                timeout: msg.timeout,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: state.core_contract.clone(),
            new_code_id: msg.core_code_id,
            msg: to_json_binary(&drop_staking_base::msg::core::MigrateMsg {
                lsm_share_bond_provider_contract: lsm_share_bond_provider_contract.to_string(),
                native_bond_provider_contract: native_bond_provider_contract.to_string(),
                factory_contract: env.contract.address.to_string(),
            })?,
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: state.distribution_contract.clone(),
            new_code_id: msg.distribution_code_id,
            msg: to_json_binary(&drop_staking_base::msg::distribution::MigrateMsg {})?,
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: state.rewards_pump_contract.clone(),
            new_code_id: msg.pump_code_id,
            msg: to_json_binary(&drop_staking_base::msg::pump::MigrateMsg {})?,
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: msg.unbonding_pump_contract.to_string(),
            new_code_id: msg.pump_code_id,
            msg: to_json_binary(&drop_staking_base::msg::pump::MigrateMsg {})?,
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: state.strategy_contract.clone(),
            new_code_id: msg.strategy_code_id,
            msg: to_json_binary(&drop_staking_base::msg::strategy::MigrateMsg {})?,
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: state.validators_set_contract.clone(),
            new_code_id: msg.validators_set_code_id,
            msg: to_json_binary(&drop_staking_base::msg::validatorset::MigrateMsg {})?,
        }));

        messages.push(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: state.puppeteer_contract.clone(),
            new_code_id: msg.puppeteer_code_id,
            msg: to_json_binary(&drop_staking_base::msg::puppeteer::MigrateMsg {
                native_bond_provider: native_bond_provider_contract.to_string(),
                factory_contract: env.contract.address.to_string(),
                allowed_senders: vec![
                    lsm_share_bond_provider_contract.to_string(),
                    native_bond_provider_contract.to_string(),
                    state.core_contract.to_string(),
                    env.contract.address.to_string(),
                ],
            })?,
        }));

        let splitter_config = deps
            .querier
            .query_wasm_smart::<drop_staking_base::state::splitter::Config>(
                &state.splitter_contract,
                &drop_staking_base::msg::splitter::QueryMsg::Config {},
            )?;

        messages.push(get_proxied_message(
            state.splitter_contract.to_string(),
            drop_staking_base::msg::splitter::ExecuteMsg::UpdateConfig {
                new_config: drop_staking_base::state::splitter::Config {
                    denom: splitter_config.denom,
                    receivers: vec![
                        (
                            native_bond_provider_contract.to_string(),
                            Uint128::from(9000u128),
                        ),
                        (
                            "neutron1xm4xgfv4xz4ccv0tjvlfac5gqwjnv9zzx4l47t7ve7j2sn4k7gwqkg947d"
                                .to_string(),
                            Uint128::from(1000u128),
                        ),
                    ],
                },
            },
            vec![],
        )?);

        STATE.save(
            deps.storage,
            WITHDRAWAL_MANAGER_CONTRACT,
            &deps.api.addr_validate(&state.withdrawal_manager_contract)?,
        )?;
        STATE.save(
            deps.storage,
            WITHDRAWAL_VOUCHER_CONTRACT,
            &deps.api.addr_validate(&state.withdrawal_voucher_contract)?,
        )?;
        STATE.save(
            deps.storage,
            VALIDATORS_SET_CONTRACT,
            &deps.api.addr_validate(&state.validators_set_contract)?,
        )?;
        STATE.save(
            deps.storage,
            DISTRIBUTION_CONTRACT,
            &deps.api.addr_validate(&state.distribution_contract)?,
        )?;
        STATE.save(
            deps.storage,
            SPLITTER_CONTRACT,
            &deps.api.addr_validate(&state.splitter_contract)?,
        )?;
        STATE.save(
            deps.storage,
            REWARDS_MANAGER_CONTRACT,
            &deps.api.addr_validate(&state.rewards_manager_contract)?,
        )?;
        STATE.save(
            deps.storage,
            REWARDS_PUMP_CONTRACT,
            &deps.api.addr_validate(&state.rewards_pump_contract)?,
        )?;
        STATE.save(
            deps.storage,
            LSM_SHARE_BOND_PROVIDER_CONTRACT,
            &deps.api.addr_validate(&lsm_share_bond_provider_contract)?,
        )?;
        STATE.save(
            deps.storage,
            NATIVE_BOND_PROVIDER_CONTRACT,
            &deps.api.addr_validate(&native_bond_provider_contract)?,
        )?;
        STATE.save(
            deps.storage,
            CORE_CONTRACT,
            &deps.api.addr_validate(&state.core_contract)?,
        )?;
        STATE.save(
            deps.storage,
            PUPPETEER_CONTRACT,
            &deps.api.addr_validate(&state.puppeteer_contract)?,
        )?;
        STATE.save(
            deps.storage,
            TOKEN_CONTRACT,
            &deps.api.addr_validate(&state.token_contract)?,
        )?;
        STATE.save(
            deps.storage,
            UNBONDING_PUMP_CONTRACT,
            &deps.api.addr_validate(&msg.unbonding_pump_contract)?,
        )?;
        STATE.save(
            deps.storage,
            STRATEGY_CONTRACT,
            &deps.api.addr_validate(&state.strategy_contract)?,
        )?;

        return Ok(Response::new().add_messages(messages));
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

    Ok(())
}

fn validate_pre_instantiated_contracts(
    deps: Deps,
    env: &Env,
    pre_instantiated_contracts: &PreInstantiatedContracts,
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
    )?;

    // Validate val ref address
    if let Some(val_ref_address) = &pre_instantiated_contracts.val_ref_address {
        validate_contract_metadata(
            deps,
            env,
            val_ref_address,
            &[drop_val_ref::contract::CONTRACT_NAME],
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
    )?;

    // Validate unbonding and rewards pump contracts
    if let Some(unbonding_pump_address) = &pre_instantiated_contracts.unbonding_pump_address {
        validate_contract_metadata(
            deps,
            env,
            unbonding_pump_address,
            &[drop_pump::contract::CONTRACT_NAME],
        )?;
    }
    if let Some(rewards_pump_address) = &pre_instantiated_contracts.rewards_pump_address {
        validate_contract_metadata(
            deps,
            env,
            rewards_pump_address,
            &[drop_pump::contract::CONTRACT_NAME],
        )?;
    }

    Ok(())
}
