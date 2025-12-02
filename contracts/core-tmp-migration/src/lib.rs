use cosmwasm_std::{
    ensure, ensure_eq, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult,
};
use drop_staking_base::{
    error::core::ContractResult,
    msg::core::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::core::{unbond_batches_map, UnbondBatchStatus, LAST_PUPPETEER_RESPONSE},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

pub type MessageWithFeeResponse<T> = (CosmosMsg<T>, Option<CosmosMsg<T>>);

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
    {
        // STEP 1: switch 2 known Withdrawing batches back to Unbonding status
        let mut withdrawing_batches_ids = unbond_batches_map()
            .idx
            .status
            .prefix(UnbondBatchStatus::Withdrawing as u8)
            .range(deps.storage, None, None, Order::Ascending)
            .map(|res| res.map(|(id, _batch)| id))
            .collect::<StdResult<Vec<_>>>()?;
        withdrawing_batches_ids.sort();
        ensure_eq!(
            withdrawing_batches_ids,
            vec![65u128, 66u128],
            StdError::generic_err("Withdrawing batches are different from the expected set")
        );

        for id in withdrawing_batches_ids {
            let mut batch = unbond_batches_map().load(deps.storage, id)?;
            batch.status = UnbondBatchStatus::Unbonding;
            unbond_batches_map().save(deps.storage, id, &batch)?;
        }
    }

    {
        // STEP 2: erase last puppeteer response
        let last_puppeteer_response = LAST_PUPPETEER_RESPONSE.may_load(deps.storage)?;
        ensure!(
            last_puppeteer_response.is_some(),
            StdError::generic_err(
                "Last puppeteer response is absent, but it is expected to be present"
            )
        );

        LAST_PUPPETEER_RESPONSE.remove(deps.storage);
    }

    Ok(Response::new())
}
