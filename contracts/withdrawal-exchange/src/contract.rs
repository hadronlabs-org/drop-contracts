use cosmwasm_std::{
    attr, ensure_eq, from_json, to_json_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    Deps, DepsMut, Env, MessageInfo, Response,
};
use cw721::{Cw721ReceiveMsg, NftInfoResponse};
use drop_helpers::{answer::response, pause::pause_guard};
use drop_staking_base::{
    error::withdrawal_exchange::{ContractError, ContractResult},
    msg::{
        withdrawal_exchange::{
            ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveNftMsg,
        },
        withdrawal_voucher::Extension,
    },
    state::withdrawal_exchange::{Config, CONFIG},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
};
use std::fmt::Display;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const UNBOND_MARK: &str = "unbond";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let attrs: Vec<Attribute> = vec![
        attr("action", "instantiate"),
        attr("withdrawal_token", &msg.withdrawal_token_address),
        attr("withdrawal_voucher", &msg.withdrawal_voucher_address),
        attr("denom_prefix", &msg.denom_prefix),
    ];

    let withdrawal_token = deps.api.addr_validate(&msg.withdrawal_token_address)?;
    let withdrawal_voucher = deps.api.addr_validate(&msg.withdrawal_voucher_address)?;
    CONFIG.save(
        deps.storage,
        &Config {
            withdrawal_token_contract: withdrawal_token,
            withdrawal_voucher_contract: withdrawal_voucher,
            denom_prefix: msg.denom_prefix,
        },
    )?;

    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(
            &cw_ownable::get_ownership(deps.storage)?
                .owner
                .unwrap_or(Addr::unchecked(""))
                .to_string(),
        )?),
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            Ok(to_json_binary(&ConfigResponse {
                withdrawal_token_address: config.withdrawal_token_contract.to_string(),
                withdrawal_voucher_address: config.withdrawal_voucher_contract.to_string(),
                denom_prefix: config.denom_prefix.to_string(),
            })?)
        }
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
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::Exchange(Cw721ReceiveMsg {
            sender,
            token_id,
            msg: raw_msg,
        }) => {
            let msg: ReceiveNftMsg = from_json(raw_msg)?;
            match msg {
                ReceiveNftMsg::Withdraw { receiver } => {
                    execute_exchange(deps, env, info, sender, token_id, receiver)
                }
            }
        }
    }
}

fn execute_exchange(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    sender: String,
    token_id: String,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    pause_guard(deps.storage)?;

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

    let subdenom = build_subdenom_name(config.denom_prefix, batch_id);
    let full_denom = query_full_denom(deps.as_ref(), env.contract.address, subdenom)?;

    let to_address = receiver.unwrap_or(sender);
    let message = CosmosMsg::Bank(BankMsg::Send {
        to_address,
        amount: vec![Coin {
            denom: full_denom.denom,
            amount: voucher_extension.amount,
        }],
    });

    let attrs = vec![attr("action", "exchange_nft")];
    Ok(response("execute-exchange_nft", CONTRACT_NAME, attrs).add_message(message))
}

fn build_subdenom_name(denom_prefix: impl Display, batch_id: impl Display) -> String {
    format!("{denom_prefix}:{UNBOND_MARK}:{batch_id}")
}
