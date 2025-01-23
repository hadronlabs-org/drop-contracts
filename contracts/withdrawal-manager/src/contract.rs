use cosmwasm_std::{
    attr, ensure_eq, from_json, to_json_binary, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw721::NftInfoResponse;
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::{answer::response, is_paused};
use drop_staking_base::{
    msg::{
        withdrawal_manager::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveNftMsg},
        withdrawal_voucher::Extension,
    },
    state::{
        core::{UnbondBatch, UnbondBatchStatus},
        withdrawal_manager::{Config, Cw721ReceiveMsg, Pause, PauseType, CONFIG, PAUSE},
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
        attr("core_contract", &msg.core_contract),
        attr("voucher_contract", &msg.voucher_contract),
        attr("base_denom", &msg.base_denom),
    ];
    PAUSE.save(deps.storage, &Pause::default())?;
    CONFIG.save(
        deps.storage,
        &Config {
            core_contract: deps.api.addr_validate(&msg.core_contract)?,
            withdrawal_voucher_contract: deps.api.addr_validate(&msg.voucher_contract)?,
            base_denom: msg.base_denom,
        },
    )?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => Ok(to_json_binary(&CONFIG.load(deps.storage)?)?),
        QueryMsg::Pause {} => Ok(to_json_binary(&PAUSE.load(deps.storage)?)?),
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
            core_contract,
            voucher_contract,
            base_denom,
        } => execute_update_config(deps, info, core_contract, voucher_contract, base_denom),
        ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender,
            token_id,
            msg: raw_msg,
        }) => {
            let msg: ReceiveNftMsg = from_json(raw_msg)?;
            match msg {
                ReceiveNftMsg::Withdraw { receiver } => {
                    execute_receive_nft_withdraw(deps, info, env, sender, token_id, receiver)
                }
            }
        }
        ExecuteMsg::SetPause { pause } => execute_set_pause(deps, info, pause),
    }
}

fn execute_set_pause(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    pause: Pause,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    PAUSE.save(deps.storage, &pause)?;

    let attrs = match pause.pause {
        PauseType::Switch {
            receive_nft_withdraw,
        } => {
            vec![("receive_nft_withdraw", receive_nft_withdraw.to_string())]
        }
        PauseType::Height {
            receive_nft_withdraw,
        } => {
            vec![("receive_nft_withdraw", receive_nft_withdraw.to_string())]
        }
    };

    Ok(response("execute-set-pause", CONTRACT_NAME, attrs))
}
fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    core_contract: Option<String>,
    voucher_contract: Option<String>,
    base_denom: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;
    let mut attrs: Vec<Attribute> = vec![attr("action", "update_config")];

    if let Some(core_contract) = core_contract {
        config.core_contract = deps.api.addr_validate(&core_contract)?;
        attrs.push(attr("core_contract", core_contract));
    }
    if let Some(voucher_contract) = voucher_contract {
        config.withdrawal_voucher_contract = deps.api.addr_validate(&voucher_contract)?;
        attrs.push(attr("voucher_contract", voucher_contract));
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
    env: Env,
    sender: String,
    token_id: String,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    if is_paused!(PAUSE, deps, env, receive_nft_withdraw) {
        return Err(drop_helpers::pause::PauseError::Paused {}.into());
    }

    let mut attrs = vec![attr("action", "receive_nft")];
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        config.withdrawal_voucher_contract,
        info.sender,
        ContractError::Unauthorized {}
    );
    let voucher: NftInfoResponse<Extension> = deps.querier.query_wasm_smart(
        config.withdrawal_voucher_contract,
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
        &config.core_contract,
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
        contract_addr: config.core_contract.to_string(),
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
