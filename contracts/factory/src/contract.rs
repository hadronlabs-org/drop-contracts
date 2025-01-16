use crate::error::ContractError;
use crate::msg::OwnerQueryMsg;
use crate::state::{BondProvider, FactoryType, PreInstantiatedContracts};
use crate::{
    error::ContractResult,
    msg::{
        ExecuteMsg, Factory, InstantiateMsg, MigrateMsg, ProxyMsg, QueryMsg, UpdateConfigMsg,
        ValidatorSetMsg,
    },
    state::{State, FACTORY_TYPE, STATE},
};
use cosmwasm_std::{
    attr, from_json, instantiate2_address, to_json_binary, Addr, Binary, CodeInfoResponse,
    CosmosMsg, Deps, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::ContractVersion;
use drop_helpers::answer::response;
use drop_staking_base::state::splitter::Config as SplitterConfig;
use drop_staking_base::{
    msg::{
        core::{InstantiateMsg as CoreInstantiateMsg, QueryMsg as CoreQueryMsg},
        distribution::InstantiateMsg as DistributionInstantiateMsg,
        lsm_share_bond_provider::InstantiateMsg as LsmShareBondProviderInstantiateMsg,
        // native_bond_provider::InstantiateMsg as NativeBondProviderInstantiateMsg,
        // native_sync_bond_provider::InstantiateMsg as NativeSyncBondProviderInstantiateMsg,
        pump::InstantiateMsg as RewardsPumpInstantiateMsg,
        rewards_manager::{
            InstantiateMsg as RewardsMangerInstantiateMsg, QueryMsg as RewardsQueryMsg,
        },
        splitter::InstantiateMsg as SplitterInstantiateMsg,
        strategy::InstantiateMsg as StrategyInstantiateMsg,
        token::InstantiateMsg as TokenInstantiateMsg,
        validatorset::InstantiateMsg as ValidatorsSetInstantiateMsg,
        withdrawal_manager::{
            InstantiateMsg as WithdrawalManagerInstantiateMsg,
            QueryMsg as WithdrawalManagerQueryMsg,
        },
        withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
    },
    state::pump::PumpTimeout,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
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
    FACTORY_TYPE.save(deps.storage, &msg.factory.to_factory_type())?;

    let mut attrs = vec![
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
    let puppeteer_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.puppeteer_code_id)?;
    let rewards_manager_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.rewards_manager_code_id)?;
    let splitter_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.splitter_code_id)?;
    let rewards_pump_contract_checksum =
        get_code_checksum(deps.as_ref(), msg.code_ids.rewards_pump_code_id)?;
    // let native_bond_contract_checksum =
    //     get_code_checksum(deps.as_ref(), msg.code_ids.native_bond_provider_code_id)?;
    let salt = msg.salt.as_bytes();

    let token_address =
        instantiate2_address(&token_contract_checksum, &canonical_self_address, salt)?;
    let core_address =
        instantiate2_address(&core_contract_checksum, &canonical_self_address, salt)?;
    let puppeteer_address =
        instantiate2_address(&puppeteer_contract_checksum, &canonical_self_address, salt)?;
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
    let rewards_pump_address = instantiate2_address(
        &rewards_pump_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    // let native_bond_provider_address = instantiate2_address(
    //     &native_bond_contract_checksum,
    //     &canonical_self_address,
    //     salt,
    // )?;

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
    let distribution_contract = deps
        .api
        .addr_humanize(&distribution_calculator_address)?
        .to_string();
    let puppeteer_contract = deps.api.addr_humanize(&puppeteer_address)?.to_string();
    let rewards_manager_contract = deps
        .api
        .addr_humanize(&rewards_manager_address)?
        .to_string();
    let rewards_pump_contract = deps.api.addr_humanize(&rewards_pump_address)?.to_string();
    let splitter_contract = deps.api.addr_humanize(&splitter_address)?.to_string();

    let native_bond_provider_contract = msg.pre_instantiated_contracts.native_bond_provider_address;

    let transfer_channel_id = match &msg.factory {
        Factory::Remote {
            transfer_channel_id,
            ..
        } => transfer_channel_id.as_str(),
        Factory::Native { .. } => "N/A",
    };

    let (puppeteer_instantiate_msg_binary, lsm_share_bond_provider_contract) = match &msg.factory {
        Factory::Remote {
            sdk_version,
            code_ids,
            icq_update_period,
            port_id,
            ..
        } => {
            attrs.push(attr("sdk_version", sdk_version));

            let lsm_share_contract_checksum =
                get_code_checksum(deps.as_ref(), code_ids.lsm_share_bond_provider_code_id)?;
            let lsm_share_bond_provider_address =
                instantiate2_address(&lsm_share_contract_checksum, &canonical_self_address, salt)?;
            let lsm_share_bond_provider_contract = deps
                .api
                .addr_humanize(&lsm_share_bond_provider_address)?
                .to_string();

            let msg = drop_staking_base::msg::puppeteer::InstantiateMsg {
                allowed_senders: vec![
                    lsm_share_bond_provider_contract.to_string(),
                    native_bond_provider_contract.to_string(),
                    core_contract.to_string(),
                    env.contract.address.to_string(),
                ],
                owner: Some(env.contract.address.to_string()),
                remote_denom: msg.remote_opts.denom.to_string(),
                update_period: *icq_update_period,
                connection_id: msg.remote_opts.connection_id.to_string(),
                port_id: port_id.clone(),
                transfer_channel_id: transfer_channel_id.to_string(),
                sdk_version: sdk_version.clone(),
                timeout: msg.remote_opts.timeout.local,
                delegations_queries_chunk_size: None,
                native_bond_provider: native_bond_provider_contract.to_string(),
            };

            (
                to_json_binary(&msg)?,
                Some(lsm_share_bond_provider_contract),
            )
        }
        Factory::Native {
            distribution_module_contract,
        } => {
            let msg = drop_staking_base::msg::puppeteer_native::InstantiateMsg {
                allowed_senders: vec![
                    native_bond_provider_contract.to_string(),
                    core_contract.to_string(),
                    env.contract.address.to_string(),
                ],
                owner: Some(env.contract.address.to_string()),
                remote_denom: msg.remote_opts.denom.to_string(),
                native_bond_provider: native_bond_provider_contract.to_string(),
                distribution_module_contract: distribution_module_contract.to_string(),
            };

            (to_json_binary(&msg)?, None)
        }
    };

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
        rewards_pump_contract: rewards_pump_contract.to_string(),
        splitter_contract: splitter_contract.to_string(),
        bond_providers: msg.bond_providers,
    };
    STATE.save(deps.storage, &state)?;

    let mut msgs = vec![
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
            msg: puppeteer_instantiate_msg_binary,
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
                strategy_contract: strategy_contract.to_string(),
                withdrawal_voucher_contract: withdrawal_voucher_contract.to_string(),
                withdrawal_manager_contract: withdrawal_manager_contract.to_string(),
                base_denom: msg.base_denom.clone(),
                remote_denom: msg.remote_opts.denom.to_string(),
                pump_ica_address: None,
                validators_set_contract: validators_set_contract.to_string(),
                unbonding_period: msg.core_params.unbonding_period,
                unbonding_safe_period: msg.core_params.unbonding_safe_period,
                unbond_batch_switch_time: msg.core_params.unbond_batch_switch_time,
                idle_min_interval: msg.core_params.idle_min_interval,
                bond_limit: msg.core_params.bond_limit,
                transfer_channel_id: transfer_channel_id.to_string(),
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
                core_contract: core_contract.to_string(),
                voucher_contract: withdrawal_voucher_contract.to_string(),
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
                        native_bond_provider_contract.to_string(),
                    )?,
                    denom: msg.base_denom.to_string(),
                },
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
    ];
    if let Factory::Remote {
        code_ids,
        lsm_share_bond_params,
        reverse_transfer_channel_id,
        min_ibc_transfer,
        min_stake_amount,
        port_id,
        ..
    } = &msg.factory
    {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.code_ids.rewards_pump_code_id,
            label: get_contract_label("rewards-pump"),
            msg: to_json_binary(&RewardsPumpInstantiateMsg {
                dest_address: Some(splitter_contract.to_string()),
                dest_channel: Some(reverse_transfer_channel_id.clone()),
                dest_port: Some(port_id.clone()),
                connection_id: msg.remote_opts.connection_id.to_string(),
                refundee: None,
                timeout: PumpTimeout {
                    local: Some(msg.remote_opts.timeout.local),
                    remote: msg.remote_opts.timeout.remote,
                },
                local_denom: msg.local_denom.to_string(),
                owner: Some(env.contract.address.to_string()),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }));
        msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: code_ids.lsm_share_bond_provider_code_id,
            label: get_contract_label("lsm-share-bond-provider"),
            msg: to_json_binary(&LsmShareBondProviderInstantiateMsg {
                owner: env.contract.address.to_string(),
                core_contract: core_contract.to_string(),
                puppeteer_contract: puppeteer_contract.to_string(),
                validators_set_contract,
                port_id: port_id.clone(),
                transfer_channel_id: transfer_channel_id.to_string(),
                timeout: msg.remote_opts.timeout.local,
                lsm_min_bond_amount: lsm_share_bond_params.lsm_min_bond_amount,
                lsm_redeem_threshold: lsm_share_bond_params.lsm_redeem_threshold,
                lsm_redeem_maximum_interval: lsm_share_bond_params.lsm_redeem_max_interval,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }));
        // msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate2 {
        //     admin: Some(env.contract.address.to_string()),
        //     code_id: msg.code_ids.native_bond_provider_code_id,
        //     label: get_contract_label("native-bond-provider"),
        //     msg: to_json_binary(&NativeBondProviderInstantiateMsg {
        //         owner: env.contract.address.to_string(),
        //         base_denom: msg.base_denom.to_string(),
        //         puppeteer_contract: puppeteer_contract.to_string(),
        //         core_contract: core_contract.to_string(),
        //         strategy_contract: strategy_contract.to_string(),
        //         min_ibc_transfer: *min_ibc_transfer,
        //         min_stake_amount: *min_stake_amount,
        //         port_id: port_id.clone(),
        //         transfer_channel_id: transfer_channel_id.to_string(),
        //         timeout: msg.remote_opts.timeout.local,
        //     })?,
        //     funds: vec![],
        //     salt: Binary::from(salt),
        // }));
    }
    // else {
    //     msgs.push(CosmosMsg::Wasm(WasmMsg::Instantiate2 {
    //         admin: Some(env.contract.address.to_string()),
    //         code_id: msg.code_ids.native_bond_provider_code_id,
    //         label: get_contract_label("native-bond-provider"),
    //         msg: to_json_binary(&NativeSyncBondProviderInstantiateMsg {
    //             owner: env.contract.address.to_string(),
    //             base_denom: msg.base_denom.to_string(),
    //             puppeteer_contract: puppeteer_contract.to_string(),
    //             core_contract: core_contract.to_string(),
    //             strategy_contract: strategy_contract.to_string(),
    //         })?,
    //         funds: vec![],
    //         salt: Binary::from(salt),
    //     }));
    // }

    Ok(response("instantiate", CONTRACT_NAME, attrs).add_messages(msgs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?),
        QueryMsg::PauseInfo {} => query_pause_info(deps),
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            Ok(to_json_binary(&ownership)?)
        }
    }
}

fn query_pause_info(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;

    to_json_binary(&crate::state::PauseInfoResponse {
        core: deps
            .querier
            .query_wasm_smart(state.core_contract, &CoreQueryMsg::Pause {})?,
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
    let attrs = vec![attr("action", "pause")];
    let messages = vec![
        get_proxied_message(
            state.core_contract,
            drop_staking_base::msg::core::ExecuteMsg::SetPause(
                drop_staking_base::state::core::Pause {
                    tick: true,
                    bond: false,
                    unbond: false,
                },
            ),
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
    Ok(response("execute-pause", CONTRACT_NAME, attrs).add_messages(messages))
}

fn exec_unpause(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let state = STATE.load(deps.storage)?;
    let attrs = vec![attr("action", "unpause")];
    let messages = vec![
        get_proxied_message(
            state.core_contract,
            drop_staking_base::msg::core::ExecuteMsg::SetPause(
                drop_staking_base::state::core::Pause {
                    tick: false,
                    bond: false,
                    unbond: false,
                },
            ),
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
    Ok(response("execute-unpause", CONTRACT_NAME, attrs).add_messages(messages))
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
                if FACTORY_TYPE.load(deps.storage)? == (FactoryType::Remote {}) {
                    messages.push(get_proxied_message(
                        state.puppeteer_contract,
                        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
                            validators: validators.iter().map(|v| { v.valoper_address.to_string() }).collect()
                        },
                        info.funds,
                    )?)
                }
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

fn get_splitter_receivers(
    fee_params: Option<crate::msg::FeeParams>,
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
    let contract_owner: String = deps
        .querier
        .query_wasm_smart(contract_addr, &OwnerQueryMsg::Owner {})?;

    Ok(contract_owner)
}

pub fn validate_contract_metadata(
    deps: Deps,
    env: Env,
    contract_addr: &Addr,
    valid_names: Vec<String>,
) -> ContractResult<()> {
    let contract_version = get_contract_version(deps, contract_addr)?;

    if valid_names.contains(&contract_version.contract) {
        return Err(ContractError::InvalidContractName {
            expected: valid_names.join(";"),
            actual: contract_version.contract,
        });
    }

    let contract_config_owner = get_contract_config_owner(deps, contract_addr)?;
    if contract_config_owner != env.contract.address.to_string() {
        return Err(ContractError::InvalidContractOwner {
            expected: env.contract.address.to_string(),
            actual: contract_config_owner,
        });
    }

    let contract_info = deps.querier.query_wasm_contract_info(contract_addr)?;

    if let Some(contract_admin) = contract_info.admin {
        if contract_admin != env.contract.address {
            return Err(ContractError::InvalidContractAdmin {
                expected: env.contract.address.to_string(),
                actual: contract_admin.to_string(),
            });
        }
    } else {
        return Err(ContractError::InvalidContractAdmin {
            expected: env.contract.address.to_string(),
            actual: "None".to_string(),
        });
    }

    Ok(())
}

fn validate_pre_instantiated_contracts(
    deps: Deps,
    env: Env,
    pre_instantiated_contracts: &PreInstantiatedContracts,
) -> Result<(), ContractError> {
    if pre_instantiated_contracts
        .native_bond_provider_address
        .is_empty()
    {
        return Err(ContractError::InvalidContractAddress {
            address: pre_instantiated_contracts
                .native_bond_provider_address
                .clone(),
            contract: "native_bond_provider".to_string(),
        });
    } else {
        validate_contract_metadata(
            deps,
            env,
            &pre_instantiated_contracts.native_bond_provider_address,
            vec![
                drop_native_bond_provider::contract::CONTRACT_NAME.to_string(),
                drop_native_sync_bond_provider::contract::CONTRACT_NAME.to_string(),
            ],
        )?;
    }

    Ok(())
}
