use crate::{
    error::{ContractError, ContractResult},
    msg::{
        BondMsg, BondingResponse, BondingsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg,
        QueryMsg,
    },
    store::{
        bondings_map,
        reply::{CoreUnbond, CORE_UNBOND},
        BondingRecord, CORE_ADDRESS, LD_TOKEN, WITHDRAWAL_MANAGER_ADDRESS,
        WITHDRAWAL_VOUCHER_ADDRESS,
    },
};
use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut,
    Env, Event, MessageInfo, Order, Reply, Response, SubMsg, Uint64, WasmMsg,
};
use cw_storage_plus::Bound;
use drop_helpers::answer::response;
use drop_staking_base::msg::withdrawal_voucher::CW721ExecuteMsg as VoucherCW721ExecuteMsg;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CORE_UNBOND_REPLY_ID: u64 = 1;
pub const PAGINATION_DEFAULT_LIMIT: Uint64 = Uint64::new(100u64);

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CORE_ADDRESS.save(deps.storage, &deps.api.addr_validate(&msg.core_address)?)?;
    WITHDRAWAL_VOUCHER_ADDRESS.save(
        deps.storage,
        &deps.api.addr_validate(&msg.withdrawal_voucher_address)?,
    )?;
    WITHDRAWAL_MANAGER_ADDRESS.save(
        deps.storage,
        &deps.api.addr_validate(&msg.withdrawal_manager_address)?,
    )?;
    LD_TOKEN.save(deps.storage, &msg.ld_token)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("core_address", msg.core_address),
            attr("withdrawal_voucher", msg.withdrawal_voucher_address),
            attr("withdrawal_manager", msg.withdrawal_manager_address),
            attr("ld_token", msg.ld_token),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond(bond_msg) => match bond_msg {
            BondMsg::WithLdAssets {} => execute_bond_with_ld_assets(deps, info),
            BondMsg::WithNFT { token_id } => execute_bond_with_nft(deps, env, info, token_id),
        },
        ExecuteMsg::Unbond { token_id } => execute_unbond(deps, info, token_id),
        ExecuteMsg::Withdraw { token_id } => execute_withdraw(deps, info, token_id),
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

fn execute_bond_with_nft(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> ContractResult<Response<NeutronMsg>> {
    let deposit = info.funds;
    ensure!(!deposit.is_empty(), ContractError::DepositExpected {});

    // XXX: this code allows user to pass ld_token as a deposit. This sounds strange, but it might actually make
    //      sense to do so. Should we introduce a check that forbids it?

    bondings_map().save(
        deps.storage,
        &token_id,
        &BondingRecord {
            bonder: info.sender,
            deposit,
        },
    )?;

    let withdrawal_voucher = WITHDRAWAL_VOUCHER_ADDRESS.load(deps.storage)?;
    let msg = WasmMsg::Execute {
        contract_addr: withdrawal_voucher.into_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::withdrawal_voucher::ExecuteMsg::Custom {
                msg: VoucherCW721ExecuteMsg::TransferNft {
                    recipient: env.contract.address.into_string(),
                    token_id,
                },
            },
        )?,
        funds: vec![],
    };

    // TODO: attributes
    Ok(Response::new().add_message(msg))
}

fn execute_unbond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    token_id: String,
) -> ContractResult<Response<NeutronMsg>> {
    let bonding = bondings_map().load(deps.storage, &token_id)?;
    ensure_eq!(info.sender, bonding.bonder, ContractError::Unauthorized {});
    bondings_map().remove(deps.storage, &token_id)?;

    let withdrawal_voucher = WITHDRAWAL_VOUCHER_ADDRESS.load(deps.storage)?;

    let nft_msg: CosmosMsg<NeutronMsg> = WasmMsg::Execute {
        contract_addr: withdrawal_voucher.into_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::withdrawal_voucher::ExecuteMsg::Custom {
                msg: VoucherCW721ExecuteMsg::TransferNft {
                    recipient: info.sender.to_string(),
                    token_id,
                },
            },
        )?,
        funds: vec![],
    }
    .into();

    let deposit_msg = BankMsg::Send {
        to_address: info.sender.into_string(),
        amount: bonding.deposit,
    }
    .into();

    // TODO: attributes
    Ok(Response::new().add_messages([nft_msg, deposit_msg]))
}

fn execute_withdraw(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    token_id: String,
) -> ContractResult<Response<NeutronMsg>> {
    let bonding = bondings_map().load(deps.storage, &token_id)?;
    bondings_map().remove(deps.storage, &token_id)?;

    let withdrawal_voucher = WITHDRAWAL_VOUCHER_ADDRESS.load(deps.storage)?;
    let withdrawal_manager = WITHDRAWAL_MANAGER_ADDRESS.load(deps.storage)?;

    let withdraw_msg: CosmosMsg<NeutronMsg> = WasmMsg::Execute {
        contract_addr: withdrawal_voucher.into_string(),
        msg: to_json_binary(
            &drop_staking_base::msg::withdrawal_voucher::ExecuteMsg::Custom {
                msg: VoucherCW721ExecuteMsg::SendNft {
                    contract: withdrawal_manager.into_string(),
                    token_id,
                    msg: to_json_binary(
                        &drop_staking_base::msg::withdrawal_manager::ReceiveNftMsg::Withdraw {
                            receiver: Some(bonding.bonder.into_string()),
                        },
                    )?,
                },
            },
        )?,
        funds: vec![],
    }
    .into();

    let deposit_msg = BankMsg::Send {
        to_address: info.sender.into_string(),
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
            reply_core_unbond(deps, sender, deposit, events)
        }
        _ => unreachable!(),
    }
}

fn reply_core_unbond(
    deps: DepsMut<NeutronQuery>,
    sender: Addr,
    deposit: Vec<Coin>,
    events: Vec<Event>,
) -> ContractResult<Response<NeutronMsg>> {
    let token_id = events
        .into_iter()
        .filter(|event| event.ty == "wasm")
        .flat_map(|event| event.attributes)
        .find(|attribute| attribute.key == "token_id")
        // it is safe to use unwrap here because cw-721 always generates valid events on success
        .unwrap()
        .value;

    bondings_map().save(
        deps.storage,
        &token_id,
        &BondingRecord {
            bonder: sender,
            deposit,
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
        let (token_id, bonding) = i?;
        bondings.push(BondingResponse {
            token_id,
            bonder: bonding.bonder.into_string(),
            deposit: bonding.deposit,
        })
    }

    let next_page_key = iter
        .next()
        .transpose()?
        .map(|(token_id, _bonding)| token_id);

    Ok(to_json_binary(&BondingsResponse {
        bondings,
        next_page_key,
    })?)
}

fn query_config(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    Ok(to_json_binary(&InstantiateMsg {
        core_address: CORE_ADDRESS.load(deps.storage)?.into_string(),
        withdrawal_voucher_address: WITHDRAWAL_VOUCHER_ADDRESS.load(deps.storage)?.into_string(),
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
