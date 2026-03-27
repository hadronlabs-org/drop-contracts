use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use drop_staking_base::{
    error::core::ContractResult,
    msg::core::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::core::{unbond_batches_map, UnbondBatchStatus},
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
    for affected_batch_id in 104u128..=107u128 {
        let mut batch = unbond_batches_map().load(deps.storage, affected_batch_id)?;
        batch.status = UnbondBatchStatus::Withdrawn;
        unbond_batches_map().save(deps.storage, affected_batch_id, &batch)?;
    }

    Ok(Response::new())
}
