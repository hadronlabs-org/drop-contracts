use cosmwasm_std::{attr, Empty};
pub use cw721_base::{ContractError, MinterResponse};
use drop_helpers::answer::response;
use drop_staking_base::{
    msg::withdrawal_voucher::Extension,
    state::withdrawal_voucher::{Pause, PAUSE},
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Cw721VoucherContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty, Empty, Empty>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
    use cw721_base::ExecuteMsg as CW721ExecuteMsg;
    use drop_staking_base::msg::withdrawal_voucher::{
        ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    };

    // This makes a conscious choice on the various generics used by the contract
    #[cosmwasm_std::entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        PAUSE.save(deps.storage, &Pause::default())?;
        Cw721VoucherContract::default().instantiate(deps.branch(), env, info, msg)
    }

    #[cosmwasm_std::entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        ensure_not_paused_method(&deps, &msg)?;
        match msg {
            ExecuteMsg::Custom { msg } => {
                Cw721VoucherContract::default().execute(deps, env, info, msg)
            }
            ExecuteMsg::SetPause(pause) => execute_set_pause(deps, info, pause),
        }
    }

    #[cosmwasm_std::entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721VoucherContract::default().query(deps, env, msg)
    }

    #[cosmwasm_std::entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
        let version: semver::Version = CONTRACT_VERSION
            .parse()
            .map_err(|e: semver::Error| StdError::generic_err(e.to_string()))?;
        let storage_version: semver::Version = cw2::get_contract_version(deps.storage)?
            .version
            .parse()
            .map_err(|e: semver::Error| StdError::generic_err(e.to_string()))?;

        if storage_version < version {
            cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        }

        Ok(Response::new())
    }

    pub fn execute_set_pause(
        deps: DepsMut,
        info: MessageInfo,
        pause: Pause,
    ) -> Result<Response, cw721_base::ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;
        PAUSE.save(deps.storage, &pause)?;
        Ok(response(
            "execute-set-pause",
            CONTRACT_NAME,
            vec![attr("mint", pause.mint.to_string())],
        ))
    }

    pub fn ensure_not_paused_method(deps: &DepsMut, msg: &ExecuteMsg) -> Result<(), ContractError> {
        match msg {
            ExecuteMsg::Custom { msg } => match msg {
                CW721ExecuteMsg::Mint { .. } => {
                    if PAUSE.load(deps.as_ref().storage)?.mint {
                        Err(ContractError::Std(StdError::GenericErr {
                            msg: "method mint is paused".to_string(),
                        }))?
                    }
                    Ok(())
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }
}
