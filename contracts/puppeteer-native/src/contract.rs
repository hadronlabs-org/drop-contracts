use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use cosmwasm_std::{
    attr, ensure, to_json_binary, Addr, Attribute, BankMsg, Coin as StdCoin, CosmosMsg, Deps,
    QueryRequest, StakingMsg, StdError, Timestamp, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use drop_helpers::{answer::response, validation::validate_addresses};

use drop_puppeteer_base::{
    error::{ContractError, ContractResult},
    msg::TransferReadyBatchesMsg,
    peripheral_hook::{ReceiverExecuteMsg, ResponseHookMsg, ResponseHookSuccessMsg, Transaction},
};
use drop_staking_base::state::puppeteer_native::REWARDS_WITHDRAW_ADDR;
use drop_staking_base::{
    msg::puppeteer_native::{
        BalancesResponse, DelegationsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryExtMsg,
        QueryMsg,
    },
    state::{
        puppeteer::{Delegations, DropDelegation},
        puppeteer_native::{
            unbonding_delegations::QueryDelegatorUnbondingDelegationsResponse, Config,
            ConfigOptional, QueryDelegatorDelegationsResponse, CONFIG,
        },
    },
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    interchain_queries::v045::types::Balances,
};
use prost::Message;
use std::{env, vec};

pub const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let allowed_senders = validate_addresses(
        deps.as_ref().into_empty(),
        msg.allowed_senders.as_ref(),
        None,
    )?;
    let owner = deps
        .api
        .addr_validate(&msg.owner.unwrap_or(info.sender.to_string()))?
        .to_string();

    let config = &Config {
        remote_denom: msg.remote_denom,
        allowed_senders: allowed_senders.clone(),
        native_bond_provider: deps.api.addr_validate(&msg.native_bond_provider)?,
        distribution_module_contract: deps.api.addr_validate(&msg.distribution_module_contract)?,
    };

    let attrs: Vec<Attribute> = vec![
        attr("owner", &owner),
        attr("remote_denom", &config.remote_denom),
        attr("native_bond_provider", &config.native_bond_provider),
        attr(
            "allowed_senders",
            allowed_senders
                .into_iter()
                .map(|addr| addr.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ),
    ];

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;
    CONFIG.save(deps.storage, config)?;

    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Extension { msg } => match msg {
            QueryExtMsg::Delegations {} => query_delegations(deps, env),
            QueryExtMsg::Balances {} => query_balances(deps, env),
            // QueryExtMsg::NonNativeRewardsBalances {} => {
            //     query_non_native_rewards_balances(deps, env)
            // }
            QueryExtMsg::UnbondingDelegations {} => query_unbonding_delegations(deps, env),
        },
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Transactions {} => query_transactions(),
        QueryMsg::Ownership {} => {
            let owner = cw_ownable::get_ownership(deps.storage)?;
            to_json_binary(&owner).map_err(ContractError::Std)
        }
    }
}

fn query_config(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_transactions() -> ContractResult<Binary> {
    Ok(to_json_binary::<[(); 0]>(&[])?)
}

fn query_delegations(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let mut key = vec![];
    let mut total_delegations = vec![];

    loop {
        let res: StdResult<QueryDelegatorDelegationsResponse> =
            deps.querier.query(&QueryRequest::Stargate {
                path: "/cosmos.staking.v1beta1.Query/DelegatorDelegations".to_string(),
                data:
                    cosmos_sdk_proto::cosmos::staking::v1beta1::QueryDelegatorDelegationsRequest {
                        delegator_addr: env.contract.address.to_string(),
                        pagination: Some(PageRequest {
                            key,
                            limit: 500,
                            ..Default::default()
                        }),
                    }
                    .encode_to_vec()
                    .into(),
            });

        if res.is_err() {
            return Ok(to_json_binary(&DelegationsResponse {
                delegations: Delegations {
                    delegations: vec![],
                },
                remote_height: 0,
                local_height: 0,
                timestamp: Timestamp::default(),
            })?);
        } else {
            let delegations_response = res.unwrap(); // unwrap is safe bc we know that it's not an error

            let delegations: Vec<DropDelegation> = delegations_response
                .delegation_responses
                .into_iter()
                .map(Into::into)
                .collect();

            total_delegations.extend(delegations);

            if delegations_response.pagination.next_key.is_none() {
                break;
            } else {
                key = delegations_response.pagination.next_key.unwrap();
            }
        }
    }

    let delegations = Delegations {
        delegations: total_delegations,
    };

    Ok(to_json_binary(&DelegationsResponse {
        delegations,
        remote_height: env.block.height,
        local_height: env.block.height,
        timestamp: env.block.time,
    })?)
}

fn query_balances(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let balance = deps
        .querier
        .query_balance(env.contract.address, config.remote_denom)?;
    Ok(to_json_binary(&BalancesResponse {
        balances: Balances {
            coins: vec![balance],
        },
        remote_height: env.block.height,
        local_height: env.block.height,
        timestamp: env.block.time,
    })?)
}

fn query_unbonding_delegations(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let mut key = vec![];
    let mut total_undelegations = vec![];
    loop {
        let unbonding_response: QueryDelegatorUnbondingDelegationsResponse = deps
        .querier
        .query(&QueryRequest::Stargate {
        path: "/cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations".to_string(),
        data:
            cosmos_sdk_proto::cosmos::staking::v1beta1::QueryDelegatorUnbondingDelegationsRequest {
                delegator_addr: env.contract.address.to_string(),
                pagination: Some(PageRequest {
                    key,
                    limit: 500,
                    ..Default::default()
                }),
            }
            .encode_to_vec()
            .into(),
    })?;

        let unbonding_delegations: Vec<drop_puppeteer_base::state::UnbondingDelegation> =
            unbonding_response.clone().try_into()?;
        total_undelegations.extend(unbonding_delegations);

        if unbonding_response.pagination.next_key.is_none() {
            break;
        } else {
            key = unbonding_response.pagination.next_key.unwrap();
        }
    }

    to_json_binary(&total_undelegations).map_err(ContractError::Std)
}

// fn query_non_native_rewards_balances(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
//     let config: Config = CONFIG.load(deps.storage)?;

//     let balances = deps
//         .querier
//         .query_all_balances(env.contract.address.to_string())?;

//     let balances_without_native = balances
//         .into_iter()
//         .filter(|b| b.denom != config.remote_denom)
//         .collect::<Vec<_>>();

//     to_json_binary(&BalancesResponse {
//         balances: Balances {
//             coins: balances_without_native,
//         },
//         remote_height: env.block.height,
//         local_height: env.block.height,
//         timestamp: env.block.time,
//     })
//     .map_err(ContractError::Std)
// }

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Delegate { items, reply_to } => {
            execute_delegate(deps.as_ref(), env, info, items, reply_to)
        }
        ExecuteMsg::Undelegate {
            items,
            batch_id,
            reply_to,
        } => execute_undelegate(deps, env, info, items, batch_id, reply_to),
        ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators,
            transfer,
            reply_to,
        } => execute_claim_rewards_and_optionaly_transfer(
            deps, env, info, validators, transfer, reply_to,
        ),
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            let attrs = vec![attr("action", "update_ownership")];
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response("update_ownership", CONTRACT_NAME, attrs))
        }
        ExecuteMsg::SetupProtocol {
            rewards_withdraw_address,
        } => execute_setup_protocol(deps, env, info, rewards_withdraw_address),
        ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery { validators: _ } => {
            Ok(Response::default())
        }
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(remote_denom) = new_config.remote_denom {
        config.remote_denom = remote_denom.clone();
        attrs.push(attr("remote_denom", remote_denom))
    }

    if let Some(allowed_senders) = new_config.allowed_senders {
        let allowed_senders =
            validate_addresses(deps.as_ref().into_empty(), allowed_senders.as_ref(), None)?;
        attrs.push(attr("allowed_senders", allowed_senders.len().to_string()));
        config.allowed_senders = allowed_senders
    }

    if let Some(native_bond_provider) = new_config.native_bond_provider {
        config.native_bond_provider = deps.api.addr_validate(&native_bond_provider)?;
        attrs.push(attr("native_bond_provider", native_bond_provider));
    }

    if let Some(distribution_module_contract) = new_config.distribution_module_contract {
        config.distribution_module_contract =
            deps.api.addr_validate(&distribution_module_contract)?;
        attrs.push(attr(
            "distribution_module_contract",
            distribution_module_contract,
        ));
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(response("config_update", CONTRACT_NAME, attrs))
}

fn execute_delegate(
    deps: Deps<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    cw_utils::must_pay(&info, &config.remote_denom)?;
    cw_utils::one_coin(&info)?;

    let amount_attached = info.funds.last().unwrap().amount;

    let amount_to_stake: Uint128 = items.iter().map(|(_, amount)| *amount).sum();

    ensure!(
        amount_attached >= amount_to_stake,
        ContractError::InvalidFunds {
            reason: "not enough funds to stake".to_string()
        }
    );

    let attrs = vec![
        attr("action", "stake"),
        attr("amount_to_stake", amount_to_stake.to_string()),
    ];

    let mut messages = vec![];
    for (validator, amount) in items.clone() {
        let delegate_msg = CosmosMsg::Staking(StakingMsg::Delegate {
            validator: validator.clone(),
            amount: StdCoin {
                denom: config.remote_denom.to_string(),
                amount,
            },
        });

        messages.push(delegate_msg);
    }

    deps.api.debug(&format!(
        "WASMDEBUG: json: {request:?}",
        request = to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
            ResponseHookMsg::Success(ResponseHookSuccessMsg {
                transaction: Transaction::Stake {
                    amount: amount_to_stake
                },
                local_height: env.block.height,
                remote_height: env.block.height,
            },)
        ))?
    ));

    if !reply_to.is_empty() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to.clone(),
            msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
                ResponseHookMsg::Success(ResponseHookSuccessMsg {
                    transaction: Transaction::Stake {
                        amount: amount_to_stake,
                    },
                    local_height: env.block.height,
                    remote_height: env.block.height,
                }),
            ))?,
            funds: vec![],
        }));
    }

    Ok(response("stake", CONTRACT_NAME, attrs).add_messages(messages))
}

fn execute_setup_protocol(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    rewards_withdraw_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    let config: Config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    let rewards_withdraw_addr = deps.api.addr_validate(&rewards_withdraw_address)?; // Splitter contract
    REWARDS_WITHDRAW_ADDR.save(deps.storage, &rewards_withdraw_addr)?;

    Ok(response(
        "execute_setup_protocol",
        CONTRACT_NAME,
        [("rewards_withdraw_address", rewards_withdraw_address)],
    ))
}

fn execute_claim_rewards_and_optionaly_transfer(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    validators: Vec<String>,
    transfer: Option<TransferReadyBatchesMsg>,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.addr_validate(&reply_to)?;
    let config: Config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;

    let mut messages = vec![];
    if let Some(transfer) = transfer.clone() {
        let send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: transfer.recipient, // Should send to withdrawal manager
            amount: vec![StdCoin {
                amount: transfer.amount,
                denom: config.remote_denom.to_string(),
            }],
        });

        messages.push(send_msg);
    }

    let rewards_withdraw_addr = REWARDS_WITHDRAW_ADDR.load(deps.storage)?;

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.distribution_module_contract.to_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::neutron_distribution_mock::ExecuteMsg::ClaimRewards {
                receiver: Some(rewards_withdraw_addr.to_string()),
            },
        )?,
        funds: vec![],
    }));

    if !reply_to.is_empty() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to.clone(),
            msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
                ResponseHookMsg::Success(ResponseHookSuccessMsg {
                    transaction: Transaction::ClaimRewardsAndOptionalyTransfer {
                        interchain_account_id: env.contract.address.to_string(),
                        validators,
                        denom: config.remote_denom.to_string(),
                        transfer,
                    },
                    local_height: env.block.height,
                    remote_height: env.block.height,
                }),
            ))?,
            funds: vec![],
        }));
    }

    Ok(Response::default().add_messages(messages))
}

fn execute_undelegate(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    items: Vec<(String, Uint128)>,
    batch_id: u128,
    reply_to: String,
) -> ContractResult<Response<NeutronMsg>> {
    deps.api.addr_validate(&reply_to)?;
    let config: Config = CONFIG.load(deps.storage)?;
    validate_sender(&config, &info.sender)?;
    let mut messages = vec![];
    for (validator, amount) in items.clone() {
        let delegate_msg = CosmosMsg::Staking(StakingMsg::Undelegate {
            validator: validator.clone(),
            amount: StdCoin {
                denom: config.remote_denom.to_string(),
                amount,
            },
        });

        messages.push(delegate_msg);
    }

    if !reply_to.is_empty() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: reply_to.clone(),
            msg: to_json_binary(&ReceiverExecuteMsg::PeripheralHook(
                ResponseHookMsg::Success(ResponseHookSuccessMsg {
                    transaction: Transaction::Undelegate {
                        interchain_account_id: env.contract.address.to_string(),
                        items,
                        denom: config.remote_denom.to_string(),
                        batch_id,
                    },
                    local_height: env.block.height,
                    remote_height: env.block.height,
                }),
            ))?,
            funds: vec![],
        }));
    }

    Ok(Response::default().add_messages(messages))
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

fn validate_sender(config: &Config, sender: &Addr) -> StdResult<()> {
    if config.allowed_senders.contains(sender) {
        Ok(())
    } else {
        Err(StdError::generic_err("Sender is not allowed"))
    }
}
