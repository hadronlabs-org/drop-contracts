use cosmwasm_std::{ensure_eq, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError};
use drop_staking_base::{
    error::core::ContractResult,
    msg::core::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::core::unbond_batches_map,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(_deps: Deps<NeutronQuery>, _env: Env, _msg: QueryMsg) -> ContractResult<Binary> {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let affected_batch_id = 72u128;

    let mut batch = unbond_batches_map().load(deps.storage, affected_batch_id)?;
    ensure_eq!(
        batch.expected_release_time,
        1766756416,
        StdError::generic_err("Expected release time was already changed before us, aborting now!")
    );

    batch.expected_release_time = 1766155105; // 2025-12-19T14:38:24.905313525Z
    unbond_batches_map().save(deps.storage, affected_batch_id, &batch)?;

    Ok(Response::new())
}
