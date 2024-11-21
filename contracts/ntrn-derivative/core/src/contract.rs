use crate::{
    error::ContractResult,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, BASE_DENOM, CONFIG, DENOM, SALT, UNBOND_ID},
};
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Attribute, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, WasmMsg,
};
use drop_helpers::answer::response;
use drop_staking_base::{
    msg::ntrn_derivative::withdrawal_voucher::ExecuteMsg as WithdrawalVoucherExecuteMsg,
    msg::ntrn_derivative::withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
    state::ntrn_derivative::withdrawal_voucher::Metadata as VoucherMetadata,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    let cosmwasm_std::CodeInfoResponse { checksum, .. } = deps
        .querier
        .query_wasm_code_info(msg.withdrawal_voucher_code_id)?;
    let canonical_self_address = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let voucher_addr = instantiate2_address(&checksum, &canonical_self_address, SALT.as_bytes())?;
    CONFIG.save(
        deps.storage,
        &Config {
            unbonding_period: msg.unbonding_period,
            withdrawal_voucher: deps.api.addr_humanize(&voucher_addr)?,
        },
    )?;
    Ok(
        response("instantiate", CONTRACT_NAME, Vec::<Attribute>::new()).add_message(
            CosmosMsg::Wasm(WasmMsg::Instantiate2 {
                admin: Some(env.contract.address.to_string()),
                code_id: msg.withdrawal_voucher_code_id,
                label: get_contract_label("rewards-manager"),
                msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
                    name: "Drop NTRN Voucher".to_string(),
                    symbol: "DROPV".to_string(),
                    minter: env.contract.address.to_string(),
                })?,
                funds: vec![],
                salt: Binary::from(SALT.as_bytes()),
            }),
        ),
    )
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            Ok(to_json_binary(&ownership)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
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
        ExecuteMsg::Bond { receiver } => execute_bond(deps, env, info, receiver),
        ExecuteMsg::Unbond { receiver } => execute_unbond(deps, env, info, receiver),
    }
}

fn execute_bond(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let amount = cw_utils::may_pay(&info, BASE_DENOM)?;
    let receiver = receiver
        .map(|a| deps.api.addr_validate(&a))
        .unwrap_or_else(|| Ok(info.sender))?;
    let dntrn_denom = DENOM.load(deps.storage)?;
    let msg = NeutronMsg::submit_mint_tokens(dntrn_denom, amount, receiver);
    Ok(Response::new()
        .add_attribute("action", "bond")
        .add_attribute("amount", amount.to_string())
        .add_message(msg))
}

fn execute_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let dntrn_denom = DENOM.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let amount_to_withdraw = cw_utils::may_pay(&info, &dntrn_denom)?;
    let receiver = receiver
        .map(|a| deps.api.addr_validate(&a))
        .unwrap_or_else(|| Ok(info.sender))?;
    let extension = VoucherMetadata {
        description: Some("Withdrawal voucher".into()),
        name: "dNTRN voucher".to_string(),
        amount: amount_to_withdraw,
        release_at: env.block.time.seconds() + config.unbonding_period,
        receiver: receiver.to_string(),
    };
    let unbond_id = UNBOND_ID.update(deps.storage, |a| StdResult::Ok(a + 1))?;
    Ok(Response::<NeutronMsg>::new()
        .add_message(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.withdrawal_voucher.to_string(),
            msg: to_json_binary(&WithdrawalVoucherExecuteMsg::Mint {
                token_id: unbond_id.to_string(),
                owner: receiver.to_string(),
                token_uri: None,
                extension: Some(extension),
            })?,
            funds: vec![],
        }))
        .add_message(NeutronMsg::submit_burn_tokens(
            dntrn_denom,
            amount_to_withdraw,
        ))
        .add_attribute("action", "unbond")
        .add_attribute("amount", amount_to_withdraw.to_string()))
}

fn get_contract_label(base: &str) -> String {
    format!("drop-staking-{}", base)
}
