use cosmwasm_std::Empty;
pub use cw721_base::{ContractError, MinterResponse};
use crate::msg::{Extension, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Cw721VoucherContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty, Empty, Empty>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

    // This makes a conscious choice on the various generics used by the contract
    #[cosmwasm_std::entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Cw721VoucherContract::default().instantiate(deps.branch(), env, info, msg)
    }

    #[cosmwasm_std::entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Cw721VoucherContract::default().execute(deps, env, info, msg)
    }

    #[cosmwasm_std::entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721VoucherContract::default().query(deps, env, msg)
    }

    #[cosmwasm_std::entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
        let contract_version_metadata = cw2::get_contract_version(deps.storage)?;
        let storage_contract_name = contract_version_metadata.contract.as_str();
        if storage_contract_name != CONTRACT_NAME {
            return Err(StdError::generic_err(format!(
                "Can't migrate from {} to {}",
                storage_contract_name, CONTRACT_NAME
            ))
            .into());
        }

        let storage_version: semver::Version = contract_version_metadata
            .version
            .parse()
            .map_err(|e: semver::Error| StdError::generic_err(e.to_string()))?;
        let version: semver::Version = CONTRACT_VERSION
            .parse()
            .map_err(|e: semver::Error| StdError::generic_err(e.to_string()))?;

        if storage_version < version {
            cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        }

        Ok(Response::new())
    }
}
