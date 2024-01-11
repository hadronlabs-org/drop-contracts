use crate::{
    error::ContractResult,
    msg::{CallbackMsg, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, State, CONFIG, STATE},
};
use cosmwasm_std::{
    attr, entry_point, instantiate2_address, to_json_binary, Binary, CodeInfoResponse, CosmosMsg,
    Deps, DepsMut, Env, HexBinary, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;

use lido_staking_base::{
    helpers::answer::response,
    msg::core::{ExecuteMsg as CoreExecuteMsg, InstantiateMsg as CoreInstantiateMsg},
    msg::token::{
        ConfigResponse as TokenConfigResponse, InstantiateMsg as TokenInstantiateMsg,
        QueryMsg as TokenQueryMsg,
    },
    msg::withdrawal_manager::InstantiateMsg as WithdrawalManagerInstantiateMsg,
    msg::withdrawal_voucher::InstantiateMsg as WithdrawalVoucherInstantiateMsg,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    NeutronResult,
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            salt: msg.salt.to_string(),
            token_code_id: msg.token_code_id,
            core_code_id: msg.core_code_id,
            withdrawal_voucher_code_id: msg.withdrawal_voucher_code_id,
            withdrawal_manager_code_id: msg.withdrawal_manager_code_id,
            owner: info.sender.to_string(),
            subdenom: msg.subdenom.to_string(),
        },
    )?;

    let attrs = vec![
        attr("salt", msg.salt),
        attr("token_code_id", msg.token_code_id.to_string()),
        attr("core_code_id", msg.core_code_id.to_string()),
        attr(
            "withdrawal_voucher_code_id",
            msg.withdrawal_voucher_code_id.to_string(),
        ),
        attr(
            "withdrawal_manager_code_id",
            msg.withdrawal_manager_code_id.to_string(),
        ),
        attr("owner", info.sender),
        attr("subdenom", msg.subdenom),
    ];
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Init { base_denom } => execute_init(deps, env, info, base_denom),
        ExecuteMsg::Callback(msg) => match msg {
            CallbackMsg::PostInit {} => execute_post_init(deps, env, info),
        },
    }
}

fn execute_init(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    base_denom: String,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let canonical_self_address = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let mut attrs = vec![
        attr("action", "init"),
        attr("owner", config.owner),
        attr("base_denom", &base_denom),
    ];

    let token_contract_checksum = get_code_checksum(deps.as_ref(), config.token_code_id)?;
    let core_contract_checksum = get_code_checksum(deps.as_ref(), config.core_code_id)?;
    let withdrawal_voucher_contract_checksum =
        get_code_checksum(deps.as_ref(), config.withdrawal_voucher_code_id)?;
    let withdrawal_manager_contract_checksum =
        get_code_checksum(deps.as_ref(), config.withdrawal_manager_code_id)?;
    let salt = config.salt.as_bytes();

    let token_address =
        instantiate2_address(&token_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("token_address", token_address.to_string()));
    let core_address =
        instantiate2_address(&core_contract_checksum, &canonical_self_address, salt)?;
    attrs.push(attr("core_address", core_address.to_string()));
    let withdrawal_voucher_address = instantiate2_address(
        &withdrawal_voucher_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    attrs.push(attr(
        "withdrawal_voucher_address",
        withdrawal_voucher_address.to_string(),
    ));
    let withdrawal_manager_address = instantiate2_address(
        &withdrawal_manager_contract_checksum,
        &canonical_self_address,
        salt,
    )?;
    attrs.push(attr(
        "withdrawal_manager_address",
        withdrawal_manager_address.to_string(),
    ));

    let core_contract = deps.api.addr_humanize(&core_address)?.to_string();
    let token_contract = deps.api.addr_humanize(&token_address)?.to_string();
    let withdrawal_voucher_contract = deps
        .api
        .addr_humanize(&withdrawal_voucher_address)?
        .to_string();
    let withdrawal_manager_contract = deps
        .api
        .addr_humanize(&withdrawal_manager_address)?
        .to_string();
    let state = State {
        token_contract: token_contract.to_string(),
        core_contract: core_contract.to_string(),
        withdrawal_voucher_contract: withdrawal_voucher_contract.to_string(),
        withdrawal_manager_contract: withdrawal_manager_contract.to_string(),
    };

    STATE.save(deps.storage, &state)?;
    let msgs = vec![
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.token_code_id,
            label: get_contract_label("token"),
            msg: to_json_binary(&TokenInstantiateMsg {
                core_address: core_contract.to_string(),
                subdenom: config.subdenom,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.core_code_id,
            label: get_contract_label("core"),
            msg: to_json_binary(&CoreInstantiateMsg {
                token_contract: token_contract.to_string(),
                puppeteer_contract: "".to_string(),
                strategy_contract: "".to_string(),
                withdrawal_voucher_contract: withdrawal_voucher_contract.to_string(),
                withdrawal_manager_contract: withdrawal_manager_contract.to_string(),
                base_denom: base_denom.to_string(),
                owner: env.contract.address.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.withdrawal_voucher_code_id,
            label: get_contract_label("withdrawal-voucher"),
            msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
                name: "Lido Voucher".to_string(),
                symbol: "LDOV".to_string(),
                minter: core_contract.to_string(),
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Instantiate2 {
            admin: Some(env.contract.address.to_string()),
            code_id: config.withdrawal_manager_code_id,
            label: get_contract_label("withdrawal-manager"),
            msg: to_json_binary(&WithdrawalManagerInstantiateMsg {
                core_contract: core_contract.to_string(),
                voucher_contract: withdrawal_voucher_contract.to_string(),
                owner: env.contract.address.to_string(),
                base_denom,
            })?,
            funds: vec![],
            salt: Binary::from(salt),
        }),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&ExecuteMsg::Callback(CallbackMsg::PostInit {}))?,
            funds: vec![],
        }),
    ];

    Ok(response("execute-init", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn execute_post_init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let attrs = vec![attr("action", "post_init")];
    let state = STATE.load(deps.storage)?;
    let token_config: TokenConfigResponse = deps
        .querier
        .query_wasm_smart(state.token_contract, &TokenQueryMsg::Config {})?;
    let core_update_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.core_contract,
        msg: to_json_binary(&CoreExecuteMsg::UpdateConfig {
            token_contract: None,
            puppeteer_contract: None,
            strategy_contract: None,
            owner: None,
            ld_denom: Some(token_config.denom),
        })?,
        funds: vec![],
    });
    Ok(response("execute-post_init", CONTRACT_NAME, attrs).add_message(core_update_msg))
}

fn get_code_checksum(deps: Deps, code_id: u64) -> NeutronResult<HexBinary> {
    let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;
    Ok(checksum)
}

fn get_contract_label(base: &str) -> String {
    format!("LIDO-staking-{}", base)
}
