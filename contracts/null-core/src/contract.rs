use crate::error::ContractResult;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{NEXT_MESSAGE_ID, SAVED_MESSAGES};
use cosmwasm_std::{
    attr, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, Uint64,
};
use drop_helpers::answer::response;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response> {
    NEXT_MESSAGE_ID.save(deps.storage, &Uint64::zero())?;
    Ok(response::<(&str, &str), _>(
        "instantiate",
        CONTRACT_NAME,
        [],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::TotalSavedMessages {} => {
            Ok(to_json_binary(&NEXT_MESSAGE_ID.load(deps.storage)?)?)
        }
        QueryMsg::SavedMessage { id } => Ok(to_json_binary(
            &SAVED_MESSAGES.load(deps.storage, id.u64())?,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::PeripheralHook(msg) => {
            let mut next_message_id = NEXT_MESSAGE_ID.load(deps.storage)?;
            let attrs = [attr("message_id", next_message_id.to_string())];

            SAVED_MESSAGES.save(
                deps.storage,
                next_message_id.u64(),
                &(info.sender.into_string(), *msg),
            )?;

            next_message_id += Uint64::one();
            NEXT_MESSAGE_ID.save(deps.storage, &next_message_id)?;

            Ok(response("execute_peripheral_hook", CONTRACT_NAME, attrs))
        }
    }
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
