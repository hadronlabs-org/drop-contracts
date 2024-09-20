use crate::{
    error::ContractResult,
    msg::{ExecuteMsg, InstantiateMsg, NftState, QueryMsg},
    state::{Config, CONFIG},
};
use cosmwasm_std::{attr, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw721::AllNftInfoResponse;
use drop_helpers::answer::response;
use drop_staking_base::{
    msg::{
        core::QueryMsg as CoreQueryMsg, factory::QueryMsg as FactoryQueryMsg,
        withdrawal_voucher::QueryMsg as WithdrawalVoucherQueryMsg,
    },
    state::factory::State as FactoryState,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    CONFIG.save(
        deps.storage,
        &Config {
            factory_contract: msg.factory_contract,
        },
    )?;
    Ok(Response::default().add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?).map_err(From::from),
        QueryMsg::NftState { nft_id } => query_nft_id(deps, env, nft_id),
        QueryMsg::Ownership {} => {
            to_json_binary(&cw_ownable::get_ownership(deps.storage)?).map_err(From::from)
        }
    }
}

fn query_nft_id(deps: Deps<NeutronQuery>, _env: Env, nft_id: String) -> ContractResult<Binary> {
    let factory_state: FactoryState = deps.querier.query_wasm_smart(
        CONFIG.load(deps.storage)?.factory_contract,
        &FactoryQueryMsg::State {},
    )?;
    let nft_details: AllNftInfoResponse<drop_staking_base::msg::withdrawal_voucher::Extension> =
        match deps.querier.query_wasm_smart(
            factory_state.withdrawal_voucher_contract,
            &WithdrawalVoucherQueryMsg::AllNftInfo {
                token_id: nft_id,
                include_expired: None,
            },
        ) {
            Ok(res) => res,
            Err(_) => return Err(crate::error::ContractError::UnknownNftId {}),
        };

    let batch_id = nft_details.info.extension.unwrap().batch_id;
    let unbond_batch: drop_staking_base::state::core::UnbondBatch = deps.querier.query_wasm_smart(
        factory_state.core_contract,
        &CoreQueryMsg::UnbondBatch {
            batch_id: cosmwasm_std::Uint128::from(batch_id.parse::<u64>().unwrap()),
        },
    )?;
    let nft_status = match unbond_batch.status {
        drop_staking_base::state::core::UnbondBatchStatus::Withdrawn => NftState::Ready,
        _ => NftState::Unready,
    };
    to_json_binary(&nft_status).map_err(From::from)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(response::<(&str, &str), _>(
                "execute-update-ownership",
                CONTRACT_NAME,
                [],
            ))
        }
    }
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    msg: Config,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut attrs = vec![attr("action", "update-config")];
    let mut config = CONFIG.load(deps.storage)?;

    config.factory_contract = deps.api.addr_validate(&msg.factory_contract)?.to_string();
    attrs.push(attr("factory_contract", msg.factory_contract));

    ContractResult::Ok(Response::default().add_attributes(attrs))
}
