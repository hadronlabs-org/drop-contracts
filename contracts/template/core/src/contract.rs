#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::EXCHANGE_RATE;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    EXCHANGE_RATE.save(deps.storage, &Decimal::zero())?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateExchangeRate { exchange_rate } => {
            execute_update_exchange_rate(deps, exchange_rate)
        }
    }
}

fn execute_update_exchange_rate(
    deps: DepsMut,
    exchange_rate: Decimal,
) -> Result<Response, ContractError> {
    EXCHANGE_RATE.save(deps.storage, &exchange_rate)?;
    Ok(Response::new().add_attribute("action", "update_exchange_rate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ExchangeRate {} => Ok(to_json_binary(&EXCHANGE_RATE.load(deps.storage)?)?),
    }
}
