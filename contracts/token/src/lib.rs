use cosmwasm_std::{
    attr, ensure_eq, ensure_ne, to_json_binary, Addr, Attribute, Binary, Deps, DepsMut, Env, Event,
    MessageInfo, Reply, Response, StdError, SubMsg, Uint128,
};
use cw_storage_plus::Item;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
};

#[cfg(test)]
mod tests;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] neutron_sdk::NeutronError),

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("nothing to mint")]
    NothingToMint,

    #[error("unknown reply id: {id}")]
    UnknownReplyId { id: u64 },
}

pub type ContractResult<T> = Result<T, ContractError>;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub core: String,
    pub subdenom: String,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    Mint { amount: Uint128, receiver: String },
    Burn {},
}

#[cosmwasm_schema::cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub core: String,
    pub denom: String,
}

#[cosmwasm_schema::cw_serde]
pub enum MigrateMsg {}

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CORE: Item<Addr> = Item::new("core");
const DENOM: Item<String> = Item::new("denom");

const CREATE_DENOM_REPLY_ID: u64 = 1;

fn response<A: Into<Attribute>>(
    ty: &str,
    attrs: impl IntoIterator<Item = A>,
) -> Response<NeutronMsg> {
    Response::new().add_event(Event::new(format!("{}-{}", CONTRACT_NAME, ty)).add_attributes(attrs))
}

fn attr_coin(
    key: impl Into<String>,
    amount: impl std::fmt::Display,
    denom: impl std::fmt::Display,
) -> Attribute {
    attr(key, format!("{}{}", amount, denom))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let core = deps.api.addr_validate(&msg.core)?;
    CORE.save(deps.storage, &core)?;

    DENOM.save(deps.storage, &msg.subdenom)?;
    let create_denom_msg = SubMsg::reply_on_success(
        NeutronMsg::submit_create_denom(&msg.subdenom),
        CREATE_DENOM_REPLY_ID,
    );

    Ok(response(
        "instantiate",
        [attr("core", core), attr("subdenom", msg.subdenom)],
    )
    .add_submessage(create_denom_msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let core = CORE.load(deps.storage)?;
    ensure_eq!(info.sender, core, ContractError::Unauthorized);

    match msg {
        ExecuteMsg::Mint { amount, receiver } => mint(deps, amount, receiver),
        ExecuteMsg::Burn {} => burn(deps, info),
    }
}

fn mint(
    deps: DepsMut<NeutronQuery>,
    amount: Uint128,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    ensure_ne!(amount, Uint128::zero(), ContractError::NothingToMint);

    let denom = DENOM.load(deps.storage)?;
    let mint_msg = NeutronMsg::submit_mint_tokens(&denom, amount, &receiver);

    Ok(response(
        "execute-mint",
        [
            attr_coin("amount", amount, denom),
            attr("receiver", receiver),
        ],
    )
    .add_message(mint_msg))
}

fn burn(deps: DepsMut<NeutronQuery>, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    let denom = DENOM.load(deps.storage)?;
    let amount = cw_utils::must_pay(&info, &denom)?;

    let burn_msg = NeutronMsg::submit_burn_tokens(&denom, amount);

    Ok(response("execute-burn", [attr_coin("amount", amount, denom)]).add_message(burn_msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let core = CORE.load(deps.storage)?.into_string();
            let denom = DENOM.load(deps.storage)?;
            Ok(to_json_binary(&ConfigResponse { core, denom })?)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    Ok(Response::new())
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
            let full_denom = query_full_denom(deps.as_ref(), env.contract.address, subdenom)?;
            DENOM.save(deps.storage, &full_denom.denom)?;

            Ok(response(
                "reply-create-denom",
                [attr("denom", full_denom.denom)],
            ))
        }
        id => Err(ContractError::UnknownReplyId { id }),
    }
}
