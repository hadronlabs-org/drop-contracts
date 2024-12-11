use crate::error::{ContractError, ContractResult};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CREATE_DENOM_REPLY_ID, DENOM, EXPONENT, TOKEN_METADATA};
use cosmos_sdk_proto::cosmos::bank::v1beta1::{DenomUnit, Metadata};
use cosmwasm_std::{entry_point, to_json_binary};
use cosmwasm_std::{
    Attribute, Binary, CosmosMsg, DenomMetadata, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint128,
};
use drop_helpers::answer::response;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
    stargate::aux::create_stargate_msg,
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let create_denom_msg = SubMsg::reply_on_success(
        NeutronMsg::submit_create_denom(&msg.subdenom),
        CREATE_DENOM_REPLY_ID,
    );
    TOKEN_METADATA.save(deps.storage, &msg.token_metadata)?;
    DENOM.save(deps.storage, &msg.subdenom)?;
    EXPONENT.save(deps.storage, &msg.exponent)?;
    Ok(
        response("instantiate", CONTRACT_NAME, Vec::<Attribute>::new())
            .add_submessage(create_denom_msg),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Mint { amount } => execute_mint(deps, info, amount),
        ExecuteMsg::Burn {} => execute_burn(deps, info),
    }
}

fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let dntrn_denom = DENOM.load(deps.storage)?;
    let mint_msg = NeutronMsg::submit_mint_tokens(dntrn_denom, amount, info.sender);
    Ok(response("execute-mint", CONTRACT_NAME, Vec::<Attribute>::new()).add_message(mint_msg))
}

fn execute_burn(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    let dntrn_denom = DENOM.load(deps.storage)?;
    let burn_msg = NeutronMsg::submit_burn_tokens(dntrn_denom, info.funds[0].amount);
    Ok(response("execute-burn", CONTRACT_NAME, Vec::<Attribute>::new()).add_message(burn_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Denom {} => Ok(to_json_binary(&DENOM.load(deps.storage)?)?),
    }
}
#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.id {
        CREATE_DENOM_REPLY_ID => {
            let subdenom = DENOM.load(deps.storage)?;
            let full_denom = query_full_denom(deps.as_ref(), &env.contract.address, subdenom)?;
            let token_metadata = TOKEN_METADATA.load(deps.storage)?;
            let exponent = EXPONENT.load(deps.storage)?;
            let msg = create_set_denom_metadata_msg(
                env.contract.address.into_string(),
                full_denom.denom.clone(),
                token_metadata.clone(),
                exponent,
            );
            deps.api
                .debug(&format!("WASMDEBUG: msg: {:?}", token_metadata));
            DENOM.save(deps.storage, &full_denom.denom)?;
            TOKEN_METADATA.remove(deps.storage);
            Ok(Response::new()
                .add_attribute("full_denom", full_denom.denom)
                .add_message(msg))
        }
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn create_set_denom_metadata_msg(
    contract_address: String,
    denom: String,
    token_metadata: DenomMetadata,
    exponent: u32,
) -> CosmosMsg<NeutronMsg> {
    create_stargate_msg(
        "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata",
        neutron_sdk::proto_types::osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata {
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
                        exponent,
                        aliases: vec![],
                    },
                ],
                base: denom,
                display: token_metadata.display,
                name: token_metadata.name,
                description: token_metadata.description,
                symbol: token_metadata.symbol,
                uri: token_metadata.uri,
                uri_hash: token_metadata.uri_hash,
            }),
        },
    )
}
