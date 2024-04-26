use crate::error::{ContractError, ContractResult};
use cosmos_sdk_proto::cosmos::bank::v1beta1::{DenomUnit, Metadata};
use cosmwasm_std::{
    attr, ensure_eq, ensure_ne, entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, SubMsg, Uint128,
};
use drop_helpers::answer::{attr_coin, response};
use drop_staking_base::{
    msg::token::{ConfigResponse, DenomMetadata, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::token::{CORE_ADDRESS, DENOM, TOKEN_METADATA},
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    proto_types::osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata,
    query::token_factory::query_full_denom,
    stargate::aux::create_stargate_msg,
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CREATE_DENOM_REPLY_ID: u64 = 1;

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

    TOKEN_METADATA.save(deps.storage, &msg.token_metadata)?;

    DENOM.save(deps.storage, &msg.subdenom)?;
    let create_denom_msg = SubMsg::reply_on_success(
        NeutronMsg::submit_create_denom(&msg.subdenom),
        CREATE_DENOM_REPLY_ID,
    );

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("core_address", core), attr("subdenom", msg.subdenom)],
    )
    .add_submessage(create_denom_msg))
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
        ExecuteMsg::Mint { amount, receiver } => mint(deps, info, amount, receiver),
        ExecuteMsg::Burn {} => burn(deps, info),
        ExecuteMsg::SetTokenMetadata { token_metadata } => {
            set_token_metadata(deps, env, info, token_metadata)
        }
    }
}

fn mint(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    amount: Uint128,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    ensure_ne!(amount, Uint128::zero(), ContractError::NothingToMint);

    let core = CORE_ADDRESS.load(deps.storage)?;
    ensure_eq!(info.sender, core, ContractError::Unauthorized);

    let denom = DENOM.load(deps.storage)?;
    let mint_msg = NeutronMsg::submit_mint_tokens(&denom, amount, &receiver);

    Ok(response(
        "execute-mint",
        CONTRACT_NAME,
        [
            attr_coin("amount", amount, denom),
            attr("receiver", receiver),
        ],
    )
    .add_message(mint_msg))
}

fn burn(deps: DepsMut<NeutronQuery>, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    let core = CORE_ADDRESS.load(deps.storage)?;
    ensure_eq!(info.sender, core, ContractError::Unauthorized);

    let denom = DENOM.load(deps.storage)?;
    let amount = cw_utils::must_pay(&info, &denom)?;

    let burn_msg = NeutronMsg::submit_burn_tokens(&denom, amount);

    Ok(response(
        "execute-burn",
        CONTRACT_NAME,
        [attr_coin("amount", amount, denom)],
    )
    .add_message(burn_msg))
}

fn set_token_metadata(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    token_metadata: DenomMetadata,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let denom = DENOM.load(deps.storage)?;

    let msg = create_set_denom_metadata_msg(
        env.contract.address.into_string(),
        denom.clone(),
        token_metadata,
    );

    Ok(response(
        "execute-set-denom-metadata",
        CONTRACT_NAME,
        [attr("denom", denom)],
    )
    .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => {
            let core_address = CORE_ADDRESS.load(deps.storage)?.into_string();
            let denom = DENOM.load(deps.storage)?;
            Ok(to_json_binary(&ConfigResponse {
                core_address,
                denom,
            })?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.id {
        CREATE_DENOM_REPLY_ID => {
            let subdenom = DENOM.load(deps.storage)?;
            let full_denom = query_full_denom(deps.as_ref(), &env.contract.address, subdenom)?;
            DENOM.save(deps.storage, &full_denom.denom)?;

            let token_metadata = TOKEN_METADATA.load(deps.storage)?;
            TOKEN_METADATA.remove(deps.storage);

            let msg = create_set_denom_metadata_msg(
                env.contract.address.into_string(),
                full_denom.denom.clone(),
                token_metadata,
            );

            Ok(response(
                "reply-create-denom",
                CONTRACT_NAME,
                [attr("denom", full_denom.denom)],
            )
            .add_message(msg))
        }
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn create_set_denom_metadata_msg(
    contract_address: String,
    denom: String,
    token_metadata: DenomMetadata,
) -> CosmosMsg<NeutronMsg> {
    create_stargate_msg(
        "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata",
        MsgSetDenomMetadata {
            sender: contract_address,
            metadata: Some(Metadata {
                denom_units: vec![
                    DenomUnit {
                        denom: denom.clone(),
                        exponent: 0,
                        aliases: vec![],
                    },
                    DenomUnit {
                        denom: token_metadata.display.clone(),
                        exponent: token_metadata.exponent,
                        aliases: vec![],
                    },
                ],
                base: denom,
                display: token_metadata.display,
                name: token_metadata.name,
                description: token_metadata.description,
                symbol: token_metadata.symbol,
                uri: token_metadata.uri.unwrap_or_default(),
                uri_hash: token_metadata.uri_hash.unwrap_or_default(),
            }),
        },
    )
}
