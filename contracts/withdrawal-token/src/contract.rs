use cosmwasm_std::{
    attr, ensure_eq, ensure_ne, entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, SubMsg, Uint128,
};
use drop_helpers::answer::{attr_coin, response};
use drop_staking_base::{
    error::withdrawal_token::{ContractError, ContractResult},
    msg::withdrawal_token::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::withdrawal_token::{CORE_ADDRESS, DENOM_PREFIX, WITHDRAWAL_MANAGER_ADDRESS},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CREATE_DENOM_REPLY_ID: u64 = 1;
pub const UNBOND_MARK: &str = "unbond";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let core = deps.api.addr_validate(&msg.core_address)?;
    CORE_ADDRESS.save(deps.storage, &core)?;

    let withdrawal_manager = deps.api.addr_validate(&msg.withdrawal_manager_address)?;
    WITHDRAWAL_MANAGER_ADDRESS.save(deps.storage, &withdrawal_manager)?;

    DENOM_PREFIX.save(deps.storage, &msg.denom_prefix)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("core_address", core),
            attr("withdrawal_manager_address", withdrawal_manager),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
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
        ExecuteMsg::CreateDenom { batch_id } => create_denom(deps, info, batch_id),
        ExecuteMsg::Mint {
            amount,
            receiver,
            batch_id,
        } => mint(deps, info, env, amount, receiver, batch_id),
        ExecuteMsg::Burn { batch_id } => burn(deps, info, env, batch_id),
    }
}

fn create_denom(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    batch_id: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let core = CORE_ADDRESS.load(deps.storage)?;
    ensure_eq!(info.sender, core, ContractError::Unauthorized);

    let denom_prefix = DENOM_PREFIX.load(deps.storage)?;
    let subdenom = build_subdenom_name(denom_prefix, batch_id);

    let create_denom_msg = SubMsg::reply_on_success(
        NeutronMsg::submit_create_denom(&subdenom),
        CREATE_DENOM_REPLY_ID,
    );

    Ok(response(
        "execute-create-denom",
        CONTRACT_NAME,
        [attr("batch_id", batch_id), attr("subdenom", subdenom)],
    )
    .add_submessage(create_denom_msg))
}

fn mint(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    receiver: String,
    batch_id: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    ensure_ne!(amount, Uint128::zero(), ContractError::NothingToMint);

    let core = CORE_ADDRESS.load(deps.storage)?;
    ensure_eq!(info.sender, core, ContractError::Unauthorized);

    let denom_prefix = DENOM_PREFIX.load(deps.storage)?;
    let subdenom = build_subdenom_name(denom_prefix, batch_id);
    let full_denom = query_full_denom(deps.as_ref(), env.contract.address, subdenom)?;
    let mint_msg = NeutronMsg::submit_mint_tokens(&full_denom.denom, amount, &receiver);

    Ok(response(
        "execute-mint",
        CONTRACT_NAME,
        [
            attr_coin("amount", amount, full_denom.denom),
            attr("receiver", receiver),
        ],
    )
    .add_message(mint_msg))
}

fn burn(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    env: Env,
    batch_id: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let withdrawal_manager = WITHDRAWAL_MANAGER_ADDRESS.load(deps.storage)?;
    ensure_eq!(info.sender, withdrawal_manager, ContractError::Unauthorized);

    let denom_prefix = DENOM_PREFIX.load(deps.storage)?;
    let subdenom = build_subdenom_name(denom_prefix, batch_id);
    let full_denom = query_full_denom(deps.as_ref(), env.contract.address, subdenom)?;

    let amount = cw_utils::must_pay(&info, &full_denom.denom)?;
    let burn_msg = NeutronMsg::submit_burn_tokens(&full_denom.denom, amount);

    Ok(response(
        "execute-burn",
        CONTRACT_NAME,
        [attr_coin("amount", amount, full_denom.denom)],
    )
    .add_message(burn_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(
            &cw_ownable::get_ownership(deps.storage)?
                .owner
                .unwrap_or(Addr::unchecked(""))
                .to_string(),
        )?),
        QueryMsg::Config {} => {
            let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();
            let withdrawal_manager_address =
                WITHDRAWAL_MANAGER_ADDRESS.load(deps.storage)?.into_string();
            let denom_prefix = DENOM_PREFIX.load(deps.storage)?;
            Ok(to_json_binary(&ConfigResponse {
                core_address,
                withdrawal_manager_address,
                denom_prefix,
            })?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response<NeutronMsg>> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.id {
        CREATE_DENOM_REPLY_ID => Ok(response(
            "reply-create-denom",
            CONTRACT_NAME,
            [attr("denom", "new unbond denom")],
        )),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn build_subdenom_name(denom_prefix: String, batch_id: Uint128) -> String {
    denom_prefix + ":" + UNBOND_MARK + ":" + &batch_id.to_string()
}