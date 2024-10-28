use crate::{
    error::{ContractError, ContractResult},
    msg::{
        BondMsg, BondingResponse, BondingsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg,
        QueryMsg,
    },
    store::{
        bondings_map,
        reply::{CoreUnbond, CORE_UNBOND},
        BondingRecord, CORE_ADDRESS, LD_TOKEN, WITHDRAWAL_DENOM_PREFIX, WITHDRAWAL_MANAGER_ADDRESS,
        WITHDRAWAL_TOKEN_ADDRESS,
    },
};
use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, Event, MessageInfo, Order, Reply, Response, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw_storage_plus::Bound;
use drop_helpers::answer::response;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CORE_UNBOND_REPLY_ID: u64 = 1;
pub const PAGINATION_DEFAULT_LIMIT: Uint64 = Uint64::new(100u64);
pub const UNBOND_MARK: &str = "unbond";

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CORE_ADDRESS.save(deps.storage, &deps.api.addr_validate(&msg.core_address)?)?;
    WITHDRAWAL_TOKEN_ADDRESS.save(
        deps.storage,
        &deps.api.addr_validate(&msg.withdrawal_token_address)?,
    )?;
    WITHDRAWAL_MANAGER_ADDRESS.save(
        deps.storage,
        &deps.api.addr_validate(&msg.withdrawal_manager_address)?,
    )?;
    LD_TOKEN.save(deps.storage, &msg.ld_token)?;
    WITHDRAWAL_DENOM_PREFIX.save(deps.storage, &msg.withdrawal_denom_prefix)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("core_address", msg.core_address),
            attr("withdrawal_token", msg.withdrawal_token_address),
            attr("withdrawal_manager", msg.withdrawal_manager_address),
            attr("ld_token", msg.ld_token),
            attr("withdrawal_denom_prefix", msg.withdrawal_denom_prefix),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond(bond_msg) => match bond_msg {
            BondMsg::WithLdAssets {} => execute_bond_with_ld_assets(deps, info),
            BondMsg::WithWithdrawalDenoms { batch_id } => {
                execute_bond_with_withdrawal_denoms(deps, info, batch_id)
            }
        },
        ExecuteMsg::Unbond { batch_id } => execute_unbond(deps, info, batch_id),
        ExecuteMsg::Withdraw { batch_id, receiver } => {
            execute_withdraw(deps, info, batch_id, receiver)
        }
    }
}

fn execute_bond_with_ld_assets(
    deps: DepsMut<NeutronQuery>,
    mut info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let ld_token = LD_TOKEN.load(deps.storage)?;

    let ld_asset = info.funds.swap_remove(
        info.funds
            .iter()
            .position(|coin| coin.denom == ld_token)
            .ok_or(ContractError::LdTokenExpected {})?,
    );
    let deposit = info.funds;
    ensure!(!deposit.is_empty(), ContractError::DepositExpected {});

    CORE_UNBOND.save(
        deps.storage,
        &CoreUnbond {
            sender: info.sender,
            deposit,
        },
    )?;

    let msg = WasmMsg::Execute {
        contract_addr: CORE_ADDRESS.load(deps.storage)?.into_string(),
        msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::Unbond {})?,
        funds: vec![ld_asset],
    };

    // TODO: attributes
    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, CORE_UNBOND_REPLY_ID)))
}

fn execute_bond_with_withdrawal_denoms(
    deps: DepsMut<NeutronQuery>,
    mut info: MessageInfo,
    batch_id: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let withdrawal_denom_prefix = WITHDRAWAL_DENOM_PREFIX.load(deps.storage)?;
    let withdrawal_token_address = WITHDRAWAL_TOKEN_ADDRESS.load(deps.storage)?;
    let withdrawal_denom =
        get_full_withdrawal_denom(withdrawal_denom_prefix, withdrawal_token_address, batch_id);

    let mut withdrawal_asset = info.funds.swap_remove(
        info.funds
            .iter()
            .position(|coin| coin.denom == withdrawal_denom)
            .ok_or(ContractError::WithdrawalAssetExpected {})?,
    );
    let mut deposit = info.funds;
    ensure!(!deposit.is_empty(), ContractError::DepositExpected {});

    // XXX: this code allows user to pass ld_token as a deposit. This sounds strange, but it might actually make
    //      sense to do so. Should we introduce a check that forbids it?

    let bonding_id = get_bonding_id(&info.sender, batch_id);
    let existing_bonding = bondings_map().may_load(deps.storage, &bonding_id)?;
    if let Some(existing_bonding) = existing_bonding {
        deposit = merge_coin_vecs(existing_bonding.deposit, deposit);
        withdrawal_asset.amount += existing_bonding.withdrawal_amount;
    }
    bondings_map().save(
        deps.storage,
        &bonding_id,
        &BondingRecord {
            bonder: info.sender,
            withdrawal_amount: withdrawal_asset.amount,
            deposit,
        },
    )?;

    // TODO: attributes
    Ok(Response::new())
}

fn execute_unbond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    batch_id: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let bonding_id = get_bonding_id(&info.sender, batch_id);
    let bonding = bondings_map().load(deps.storage, &bonding_id)?;
    ensure_eq!(info.sender, bonding.bonder, ContractError::Unauthorized {});
    bondings_map().remove(deps.storage, &bonding_id)?;

    let withdrawal_denom_prefix = WITHDRAWAL_DENOM_PREFIX.load(deps.storage)?;
    let withdrawal_token_address = WITHDRAWAL_TOKEN_ADDRESS.load(deps.storage)?;
    let withdrawal_denom =
        get_full_withdrawal_denom(withdrawal_denom_prefix, withdrawal_token_address, batch_id);

    let send_assets_msg: BankMsg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: merge_coin_vecs(
            vec![Coin::new(
                bonding.withdrawal_amount.u128(),
                withdrawal_denom,
            )],
            bonding.deposit,
        ),
    };

    // TODO: attributes
    Ok(Response::new().add_message(send_assets_msg))
}

fn execute_withdraw(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    batch_id: Uint128,
    receiver: Option<Addr>,
) -> ContractResult<Response<NeutronMsg>> {
    let bonder = receiver.unwrap_or(info.sender.clone());

    let bonding_id = get_bonding_id(&bonder, batch_id);
    let bonding = bondings_map().load(deps.storage, &bonding_id)?;

    bondings_map().remove(deps.storage, &bonding_id)?;

    let withdrawal_denom_prefix = WITHDRAWAL_DENOM_PREFIX.load(deps.storage)?;
    let withdrawal_token_address = WITHDRAWAL_TOKEN_ADDRESS.load(deps.storage)?;
    let withdrawal_denom =
        get_full_withdrawal_denom(withdrawal_denom_prefix, withdrawal_token_address, batch_id);

    let withdrawal_manager = WITHDRAWAL_MANAGER_ADDRESS.load(deps.storage)?;

    let withdraw_msg: CosmosMsg<NeutronMsg> = WasmMsg::Execute {
        contract_addr: withdrawal_manager.into_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::withdrawal_manager::ExecuteMsg::ReceiveWithdrawalDenoms {
                receiver: Some(bonder.into_string()),
            },
        )?,
        funds: vec![Coin::new(
            bonding.withdrawal_amount.u128(),
            withdrawal_denom,
        )],
    }
    .into();

    let deposit_msg = BankMsg::Send {
        to_address: info.sender.clone().into_string(),
        amount: bonding.deposit,
    }
    .into();

    // TODO: attributes
    Ok(Response::new().add_messages([withdraw_msg, deposit_msg]))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    reply: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match reply.id {
        CORE_UNBOND_REPLY_ID => {
            let CoreUnbond { sender, deposit } = CORE_UNBOND.load(deps.storage)?;
            CORE_UNBOND.remove(deps.storage);
            // it is safe to use unwrap() here since this reply is only called on success
            let events = reply.result.unwrap().events;
            deps.api.debug(&format!("WASMDEBUG: {:?}", events));
            reply_core_unbond(deps, sender, deposit, events)
        }
        _ => unreachable!(),
    }
}

fn get_core_event_value(events: &[Event], key: &str) -> String {
    events
        .iter()
        .filter(|event| event.ty == "wasm-drop-withdrawal-token-execute-mint")
        .flat_map(|event| event.attributes.iter())
        .find(|attribute| attribute.key == key)
        .unwrap()
        .value
        .clone()
}

fn merge_coin_vecs(vec1: Vec<Coin>, vec2: Vec<Coin>) -> Vec<Coin> {
    let mut coin_map: HashMap<String, Uint128> = HashMap::new();

    for coin in vec1.into_iter().chain(vec2.into_iter()) {
        coin_map
            .entry(coin.denom)
            .and_modify(|e| *e += coin.amount)
            .or_insert(coin.amount);
    }

    let mut merged_coins: Vec<Coin> = coin_map
        .into_iter()
        .map(|(denom, amount)| Coin { denom, amount })
        .collect();

    merged_coins.sort_by(|a, b| a.denom.cmp(&b.denom));

    merged_coins
}

fn get_bonding_id(sender: impl Display, batch_id: impl Display) -> String {
    format!("{sender}_{batch_id}")
}

fn get_full_withdrawal_denom(
    withdrawal_denom_prefix: impl Display,
    withdrawal_token_address: impl Display,
    batch_id: Uint128,
) -> String {
    format!("factory/{withdrawal_token_address}/{withdrawal_denom_prefix}:{UNBOND_MARK}:{batch_id}")
}

fn reply_core_unbond(
    deps: DepsMut<NeutronQuery>,
    sender: Addr,
    mut deposit: Vec<Coin>,
    events: Vec<Event>,
) -> ContractResult<Response<NeutronMsg>> {
    let batch_id = get_core_event_value(&events, "batch_id");
    let str_amount = get_core_event_value(&events, "amount");

    let mut amount = Uint128::from_str(&str_amount)?;

    let bonding_id = get_bonding_id(&sender, batch_id);

    let existing_bonding = bondings_map().may_load(deps.storage, &bonding_id)?;
    if let Some(existing_bonding) = existing_bonding {
        deposit = merge_coin_vecs(existing_bonding.deposit, deposit);
        amount += existing_bonding.withdrawal_amount;
    }
    bondings_map().save(
        deps.storage,
        &bonding_id,
        &BondingRecord {
            bonder: sender,
            deposit,
            withdrawal_amount: amount,
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Bondings {
            user,
            limit,
            page_key,
        } => query_all_bondings(deps, user, limit, page_key),
        QueryMsg::Config {} => query_config(deps),
    }
}

fn query_all_bondings(
    deps: Deps<NeutronQuery>,
    user: Option<String>,
    limit: Option<Uint64>,
    page_key: Option<String>,
) -> ContractResult<Binary> {
    let user = user.map(|addr| deps.api.addr_validate(&addr)).transpose()?;
    let limit = limit.unwrap_or(PAGINATION_DEFAULT_LIMIT);
    let page_key = page_key.as_deref().map(Bound::inclusive);
    let mut iter = match user {
        None => bondings_map().range(deps.storage, page_key, None, Order::Ascending),
        Some(addr) => bondings_map().idx.bonder.prefix(addr).range(
            deps.storage,
            page_key,
            None,
            Order::Ascending,
        ),
    };

    let usize_limit = if limit <= Uint64::MAX {
        limit.u64() as usize
    } else {
        return Err(ContractError::QueryBondingsLimitExceeded {});
    };

    let mut bondings = vec![];
    for i in (&mut iter).take(usize_limit) {
        let (bonding_id, bonding) = i?;
        bondings.push(BondingResponse {
            bonding_id,
            bonder: bonding.bonder.into_string(),
            deposit: bonding.deposit,
            withdrawal_amount: bonding.withdrawal_amount,
        })
    }

    let next_page_key = iter
        .next()
        .transpose()?
        .map(|(bonding_id, _bonding)| bonding_id);

    Ok(to_json_binary(&BondingsResponse {
        bondings,
        next_page_key,
    })?)
}

fn query_config(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    Ok(to_json_binary(&InstantiateMsg {
        core_address: CORE_ADDRESS.load(deps.storage)?.into_string(),
        withdrawal_token_address: WITHDRAWAL_TOKEN_ADDRESS.load(deps.storage)?.into_string(),
        withdrawal_denom_prefix: WITHDRAWAL_DENOM_PREFIX.load(deps.storage)?,
        withdrawal_manager_address: WITHDRAWAL_MANAGER_ADDRESS.load(deps.storage)?.into_string(),
        ld_token: LD_TOKEN.load(deps.storage)?,
    })?)
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
