use cosmwasm_std::{
    attr, ensure, ensure_eq, to_json_binary, Attribute, Coin, CosmosMsg, Decimal, Deps, Reply,
    StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::{attr_coin, response};
use drop_puppeteer_base::msg::{IBCTransferReason, ReceiverExecuteMsg};
use drop_staking_base::error::native_bond_provider::{ContractError, ContractResult};
use drop_staking_base::msg::native_bond_provider::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use drop_staking_base::state::native_bond_provider::{
    Config, ConfigOptional, CONFIG, NON_STAKED_BALANCE, PUPPETEER_TRANSFER_REPLY_ID,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let puppeteer_contract = deps.api.addr_validate(&msg.puppeteer_contract)?;
    let core_contract = deps.api.addr_validate(&msg.core_contract)?;
    let config = &Config {
        puppeteer_contract: puppeteer_contract.clone(),
        core_contract: core_contract.clone(),
        base_denom: msg.base_denom.to_string(),
        min_ibc_transfer: msg.min_ibc_transfer,
    };
    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [
            attr("puppeteer_contract", puppeteer_contract.into_string()),
            attr("core_contract", core_contract.into_string()),
            attr("min_ibc_transfer", msg.min_ibc_transfer),
            attr("base_denom", msg.base_denom),
        ],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::CanBond { denom } => query_can_bond(deps, denom),
        QueryMsg::CanProcessOnIdle {} => Ok(to_json_binary(&false)?),
        QueryMsg::TokensAmount {
            coin,
            exchange_rate,
        } => query_token_amount(deps, coin, exchange_rate),
        QueryMsg::AsyncTokensAmount {} => Ok(to_json_binary(&Uint128::zero())?),
        QueryMsg::NonStakedBalance {} => query_non_staked_balance(deps, env),
        QueryMsg::AllBalance {} => query_all_balance(deps, env),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_non_staked_balance(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    Ok(to_json_binary(&(balance))?)
}

fn query_all_balance(deps: Deps<NeutronQuery>, env: Env) -> ContractResult<Binary> {
    let balance = NON_STAKED_BALANCE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let local_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), config.base_denom)?
        .amount;
    to_json_binary(&(balance + local_balance)).map_err(ContractError::Std)
}

fn query_can_bond(deps: Deps<NeutronQuery>, denom: String) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    Ok(to_json_binary(&can_bond(config.base_denom, denom))?)
}

fn query_token_amount(
    deps: Deps<NeutronQuery>,
    coin: Coin,
    exchange_rate: Decimal,
) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    if can_bond(config.base_denom, coin.denom) {
        let issue_amount = coin.amount * (Decimal::one() / exchange_rate);

        return Ok(to_json_binary(&issue_amount)?);
    }

    Err(ContractError::InvalidDenom {})
}

fn can_bond(base_denom: String, denom: String) -> bool {
    base_denom == denom
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::Bond {} => execute_bond(deps, info),
        ExecuteMsg::ProcessOnIdle {} => Err(ContractError::MessageIsNotSupported {}),
        ExecuteMsg::PuppeteerTransfer {} => execute_puppeteer_transfer(deps, env),
        ExecuteMsg::PuppeteerHook(msg) => execute_puppeteer_hook(deps, env, info, *msg),
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut state = CONFIG.load(deps.storage)?;
    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(puppeteer_contract) = new_config.puppeteer_contract {
        state.puppeteer_contract = deps.api.addr_validate(puppeteer_contract.as_ref())?;
        attrs.push(attr("puppeteer_contract", puppeteer_contract))
    }

    if let Some(core_contract) = new_config.core_contract {
        state.core_contract = deps.api.addr_validate(core_contract.as_ref())?;
        attrs.push(attr("core_contract", core_contract))
    }

    if let Some(base_denom) = new_config.base_denom {
        state.base_denom = base_denom.to_string();
        attrs.push(attr("base_denom", base_denom));
    }

    if let Some(min_ibc_transfer) = new_config.min_ibc_transfer {
        state.min_ibc_transfer = min_ibc_transfer;
        attrs.push(attr("min_ibc_transfer", min_ibc_transfer));
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let Coin { amount, denom } = cw_utils::one_coin(&info)?;
    let config = CONFIG.load(deps.storage)?;

    if denom != config.base_denom {
        return Err(ContractError::InvalidDenom {});
    }

    Ok(response(
        "bond",
        CONTRACT_NAME,
        [attr_coin("received_funds", amount.to_string(), denom)],
    ))
}

fn execute_process_on_idle(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let non_staked_balance = NON_STAKED_BALANCE.load(deps.storage)?;
    if non_staked_balance < config.min_stake_amount {
        return Err(ContractError::NotEnoughToDelegate {
            min_stake_amount: config.min_stake_amount,
            non_staked_balance,
        });
    }

    let mut attrs = vec![attr("action", "process_on_idle")];
    let mut messages: Vec<CosmosMsg<NeutronMsg>> = vec![];

    attrs.push(attr("knot", "036"));
    if let Some(lsm_msg) = get_pending_redeem_msg(deps.as_ref(), &config, &env, info.funds.clone())?
    {
        messages.push(lsm_msg);
        attrs.push(attr("knot", "037"));
        attrs.push(attr("knot", "038"));
    } else {
        attrs.push(attr("knot", "041"));
        if let Some(lsm_msg) =
            get_pending_lsm_share_msg(deps.as_ref(), &config, &env, info.funds.clone())?
        {
            messages.push(lsm_msg);
            attrs.push(attr("knot", "042"));
            attrs.push(attr("knot", "043"));
        }
    }

    Ok(
        response("update_config", CONTRACT_NAME, Vec::<Attribute>::new())
            .add_messages(messages)
            .add_attributes(attrs),
    )
}

fn execute_puppeteer_transfer(
    deps: DepsMut<NeutronQuery>,
    env: Env,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let pending_coin = deps
        .querier
        .query_balance(&env.contract.address, config.base_denom)?;
    ensure!(
        pending_coin.amount >= config.min_ibc_transfer,
        ContractError::InvalidFunds {
            reason: "amount is less than min_ibc_transfer".to_string()
        }
    );
    NON_STAKED_BALANCE.update(deps.storage, |balance| {
        StdResult::Ok(balance + pending_coin.amount)
    })?;
    let attrs = vec![
        attr("action", "puppeteer_transfer"),
        attr("pending_amount", pending_coin.amount),
    ];

    let puppeteer_transfer = SubMsg::reply_on_error(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.puppeteer_contract.to_string(),
            msg: to_json_binary(
                &drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
                    reason: IBCTransferReason::Delegate,
                    reply_to: env.contract.address.to_string(),
                },
            )?,
            funds: vec![pending_coin],
        }),
        PUPPETEER_TRANSFER_REPLY_ID,
    );

    Ok(response("puppeteer_transfer", CONTRACT_NAME, attrs).add_submessage(puppeteer_transfer))
}

fn execute_puppeteer_hook(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: drop_puppeteer_base::msg::ResponseHookMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        info.sender,
        config.puppeteer_contract,
        ContractError::Unauthorized {}
    );
    if let drop_puppeteer_base::msg::ResponseHookMsg::Success(success_msg) = msg.clone() {
        if let drop_puppeteer_base::msg::Transaction::Stake { items } = success_msg.transaction {
            let amount_to_stake: Uint128 = items.iter().map(|(_, amount)| *amount).sum();

            NON_STAKED_BALANCE.update(deps.storage, |balance| {
                StdResult::Ok(balance - amount_to_stake)
            })?;
        }
    }

    let hook_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.core_contract.to_string(),
        msg: to_json_binary(&ReceiverExecuteMsg::PuppeteerHook(msg))?,
        funds: vec![],
    });

    Ok(response(
        "execute-puppeteer_hook",
        CONTRACT_NAME,
        vec![attr("action", "puppeteer_hook")],
    )
    .add_message(hook_message))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        PUPPETEER_TRANSFER_REPLY_ID => puppeteer_transfer_reply(deps.as_ref(), msg),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn puppeteer_transfer_reply(_deps: Deps, msg: Reply) -> ContractResult<Response> {
    if let SubMsgResult::Err(err) = msg.result {
        return Err(ContractError::PuppeteerError { message: err });
    }

    Ok(Response::new())
}
