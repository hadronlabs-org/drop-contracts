use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::STATE;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use drop_staking_base::state::factory::State as FactoryState;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    STATE.save(
        deps.storage,
        &FactoryState {
            token_contract: "token_contract".to_string(),
            core_contract: "core_contract".to_string(),
            puppeteer_contract: "puppeteer_contract".to_string(),
            withdrawal_voucher_contract: "withdrawal_voucher_contract".to_string(),
            withdrawal_manager_contract: "withdrawal_manager_contract".to_string(),
            strategy_contract: "strategy_contract".to_string(),
            validators_set_contract: "validators_set_contract".to_string(),
            distribution_contract: "distribution_contract".to_string(),
            rewards_manager_contract: "rewards_manager_contract".to_string(),
            rewards_pump_contract: "rewards_pump_contract".to_string(),
            splitter_contract: "splitter_contract".to_string(),
            lsm_share_bond_provider_contract: "lsm_share_bond_provider_contract".to_string(),
            native_bond_provider_contract: "native_bond_provider_contract".to_string(),
        },
    )?;
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
        ExecuteMsg::UpdateState { state } => execute_update_state(deps, state),
    }
}

fn execute_update_state(deps: DepsMut, state: FactoryState) -> Result<Response, ContractError> {
    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("action", "update_state"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => Ok(to_json_binary(&STATE.load(deps.storage)?)?),
    }
}
