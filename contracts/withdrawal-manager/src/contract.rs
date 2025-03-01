use cosmwasm_std::{
    attr, ensure_eq, from_json, to_json_binary, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw721::NftInfoResponse;
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::{
    answer::response,
    get_contracts,
    pause::{is_paused, pause_guard, set_pause, unpause, PauseInfoResponse},
};
use drop_staking_base::{
    msg::{
        withdrawal_manager::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveNftMsg},
        withdrawal_voucher::Extension,
    },
    state::{
        core::{UnbondBatch, UnbondBatchStatus},
        withdrawal_manager::{Config, Cw721ReceiveMsg, CONFIG},
    },
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

use crate::error::{ContractError, ContractResult};
const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let attrs: Vec<Attribute> = vec![
        attr("action", "instantiate"),
        attr("factory_contract", &msg.factory_contract),
        attr("base_denom", &msg.base_denom),
    ];
    CONFIG.save(
        deps.storage,
        &Config {
            factory_contract: deps.api.addr_validate(&msg.factory_contract)?,
            base_denom: msg.base_denom,
        },
    )?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::PauseInfo {} => query_pause_info(deps),
    }
}

fn query_pause_info(deps: Deps<NeutronQuery>) -> StdResult<Binary> {
    if is_paused(deps.storage)? {
        to_json_binary(&PauseInfoResponse::Paused {})
    } else {
        to_json_binary(&PauseInfoResponse::Unpaused {})
    }
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
        ExecuteMsg::UpdateConfig {
            factory_contract,
            base_denom,
        } => execute_update_config(deps, info, factory_contract, base_denom),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender,
            token_id,
            msg: raw_msg,
        }) => {
            let msg: ReceiveNftMsg = from_json(raw_msg)?;
            match msg {
                ReceiveNftMsg::Withdraw { receiver } => {
                    execute_receive_nft_withdraw(deps, info, sender, token_id, receiver)
                }
            }
        }
        ExecuteMsg::Pause {} => exec_pause(deps, info),
        ExecuteMsg::Unpause {} => exec_unpause(deps, info),
    }
}

fn exec_pause(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    set_pause(deps.storage)?;

    Ok(response(
        "exec_pause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn exec_unpause(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    unpause(deps.storage);

    Ok(response(
        "exec_unpause",
        CONTRACT_NAME,
        Vec::<Attribute>::new(),
    ))
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    factory_contract: Option<String>,
    base_denom: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;
    let mut attrs: Vec<Attribute> = vec![attr("action", "update_config")];

    if let Some(factory_contract) = factory_contract {
        config.factory_contract = deps.api.addr_validate(&factory_contract)?;
        attrs.push(attr("factory_contract", factory_contract));
    }
    if let Some(base_denom) = base_denom {
        attrs.push(attr("base_denom", &base_denom));
        config.base_denom = base_denom;
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_receive_nft_withdraw(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    sender: String,
    token_id: String,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    pause_guard(deps.storage)?;

    let mut attrs = vec![attr("action", "receive_nft")];
    let config = CONFIG.load(deps.storage)?;
    let addrs = get_contracts!(
        deps,
        config.factory_contract,
        withdrawal_voucher_contract,
        core_contract
    );
    ensure_eq!(
        addrs.withdrawal_voucher_contract,
        info.sender,
        ContractError::Unauthorized {}
    );
    let voucher: NftInfoResponse<Extension> = deps.querier.query_wasm_smart(
        addrs.withdrawal_voucher_contract,
        &drop_staking_base::msg::withdrawal_voucher::QueryMsg::NftInfo { token_id },
    )?;
    let voucher_extension = voucher.extension.ok_or_else(|| ContractError::InvalidNFT {
        reason: "extension is not set".to_string(),
    })?;

    let batch_id =
        voucher_extension
            .batch_id
            .parse::<u128>()
            .map_err(|_| ContractError::InvalidNFT {
                reason: "invalid batch_id".to_string(),
            })?;

    let unbond_batch: UnbondBatch = deps.querier.query_wasm_smart(
        &addrs.core_contract,
        &drop_staking_base::msg::core::QueryMsg::UnbondBatch {
            batch_id: batch_id.into(),
        },
    )?;
    ensure_eq!(
        unbond_batch.status,
        UnbondBatchStatus::Withdrawn,
        ContractError::BatchIsNotWithdrawn {}
    );

    let user_share = Decimal::from_ratio(
        voucher_extension.amount,
        unbond_batch.total_dasset_amount_to_withdraw,
    );

    let payout_amount = user_share * unbond_batch.unbonded_amount.unwrap_or(Uint128::zero());
    let to_address = receiver.unwrap_or(sender);
    attrs.push(attr("batch_id", batch_id.to_string()));
    attrs.push(attr("payout_amount", payout_amount.to_string()));
    attrs.push(attr("to_address", &to_address));

    let mut messages = vec![CosmosMsg::Bank(BankMsg::Send {
        to_address,
        amount: vec![Coin {
            denom: config.base_denom,
            amount: payout_amount,
        }],
    })];

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: addrs.core_contract.to_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::core::ExecuteMsg::UpdateWithdrawnAmount {
                batch_id,
                withdrawn_amount: payout_amount,
            },
        )?,
        funds: info.funds,
    }));

    Ok(response("execute-receive_nft", CONTRACT_NAME, attrs).add_messages(messages))
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
