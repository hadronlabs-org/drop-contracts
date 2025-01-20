use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Reply, Response, StdResult, SubMsg, WasmMsg,
};
use drop_helpers::answer::response;
use drop_staking_base::{
    msg::{
        core::BondHook,
        val_ref::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, Ref},
        validatorset::{ExecuteMsg as ValidatorSetExecuteMsg, OnTopEditOperation},
    },
    state::val_ref::{CORE_ADDRESS, REFS, VALIDATORS_SET_ADDRESS},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const EDIT_ON_TOP_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
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

    let validators_set = deps.api.addr_validate(&msg.validators_set_address)?;
    VALIDATORS_SET_ADDRESS.save(deps.storage, &validators_set)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("core_address", core),
            attr("validators_set_address", validators_set),
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
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
        ExecuteMsg::BondCallback(bond_hook) => execute_bond_hook(deps, info, bond_hook),
        ExecuteMsg::UpdateConfig {
            core_address,
            validators_set_address,
        } => execute_update_config(deps, info, core_address, validators_set_address),
        ExecuteMsg::SetRefs { refs } => execute_set_refs(deps, info, refs),
    }
}

fn execute_bond_hook(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    bond_hook: BondHook,
) -> ContractResult<Response<NeutronMsg>> {
    let core = CORE_ADDRESS.load(deps.storage)?;
    if info.sender != core {
        return Err(ContractError::Unauthorized {});
    }

    let mut messages = vec![];
    let mut attrs = vec![];
    if let Some(r#ref) = bond_hook.r#ref {
        attrs.push(attr("ref", &r#ref));
        if let Some(validator_address) = REFS.may_load(deps.storage, &r#ref)? {
            attrs.push(attr("validator", &validator_address));
            let exchange_rate: Decimal = deps.querier.query_wasm_smart(
                core,
                &drop_staking_base::msg::core::QueryMsg::ExchangeRate {},
            )?;
            let on_top_increase = bond_hook.dasset_minted.mul_floor(exchange_rate);
            messages.push(SubMsg::reply_on_error(
                WasmMsg::Execute {
                    contract_addr: VALIDATORS_SET_ADDRESS.load(deps.storage)?.into_string(),
                    funds: vec![],
                    msg: to_json_binary(&ValidatorSetExecuteMsg::EditOnTop {
                        operations: vec![OnTopEditOperation::Add {
                            validator_address,
                            amount: on_top_increase,
                        }],
                    })?,
                },
                EDIT_ON_TOP_REPLY_ID,
            ));
            attrs.push(attr("on_top_increase", on_top_increase));
        } else {
            attrs.push(attr("validator", "None"));
        }
    }

    Ok(response("execute-bond-hook", CONTRACT_NAME, attrs).add_submessages(messages))
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    core_address: String,
    validators_set_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let core = deps.api.addr_validate(&core_address)?;
    CORE_ADDRESS.save(deps.storage, &core)?;

    let validators_set = deps.api.addr_validate(&validators_set_address)?;
    VALIDATORS_SET_ADDRESS.save(deps.storage, &validators_set)?;

    Ok(response(
        "execute-update-config",
        CONTRACT_NAME,
        [
            attr("core_address", core),
            attr("validators_set_address", validators_set),
        ],
    ))
}

fn execute_set_refs(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    refs: Vec<Ref>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    REFS.clear(deps.storage);
    for r#ref in &refs {
        REFS.save(deps.storage, &r#ref.r#ref, &r#ref.validator_address)?;
    }

    Ok(response(
        "execute-set-refs",
        CONTRACT_NAME,
        [attr("total_refs", refs.len().to_string())],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&cw_ownable::get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => Ok(to_json_binary(&ConfigResponse {
            core_address: CORE_ADDRESS.load(deps.storage)?.into_string(),
            validators_set_address: VALIDATORS_SET_ADDRESS.load(deps.storage)?.into_string(),
        })?),
        QueryMsg::Ref { r#ref } => {
            let validator_address = REFS.load(deps.storage, &r#ref)?;
            Ok(to_json_binary(&Ref {
                r#ref,
                validator_address,
            })?)
        }
        QueryMsg::AllRefs {} => Ok(to_json_binary(
            &REFS
                .range(deps.storage, None, None, Order::Ascending)
                .map(|r| {
                    r.map(|(r#ref, validator_address)| Ref {
                        r#ref,
                        validator_address,
                    })
                })
                .collect::<StdResult<Vec<_>>>()?,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        EDIT_ON_TOP_REPLY_ID => Ok(response(
            "reply",
            CONTRACT_NAME,
            [attr("edit_on_top_error", true.to_string())],
        )),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response<NeutronMsg>> {
    let contract_version_metadata = cw2::get_contract_version(deps.storage)?;
    let storage_contract_name = contract_version_metadata.contract.as_str();
    if storage_contract_name != CONTRACT_NAME {
        return Err(ContractError::MigrationError {
            storage_contract_name: storage_contract_name.to_string(),
            contract_name: CONTRACT_NAME.to_string(),
        });
    }

    let storage_version: semver::Version = contract_version_metadata.version.parse()?;
    let version: semver::Version = CONTRACT_VERSION.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}
