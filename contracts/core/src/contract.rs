use crate::error::{ContractError, ContractResult};
use cosmwasm_std::{
    attr, ensure_eq, ensure_ne, entry_point, to_json_binary, Attribute, BankMsg, Binary, Coin,
    CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult,
    Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::NftInfoResponse;
use lido_staking_base::helpers::answer::response;
use lido_staking_base::msg::voucher::Extension;
use lido_staking_base::msg::{
    core::{ExecuteMsg, InstantiateMsg, QueryMsg},
    token::ExecuteMsg as TokenExecuteMsg,
    voucher::ExecuteMsg as VoucherExecuteMsg,
};
use lido_staking_base::state::core::{
    Cw721ReceiveMsg, UnbondBatchStatus, UnbondItem, CONFIG, UNBOND_BATCHES, UNBOND_BATCH_ID,
};
use lido_staking_base::state::voucher::{Metadata, Trait};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::str::FromStr;
use std::vec;
const CONTRACT_NAME: &str = concat!("crates.io:lido-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &msg.clone().into())?;
    let attrs: Vec<Attribute> = vec![
        attr("token_contract", msg.token_contract),
        attr("puppeteer_contract", msg.puppeteer_contract),
        attr("strategy_contract", msg.strategy_contract),
        attr("base_denom", msg.base_denom),
        attr("owner", msg.owner),
    ];
    UNBOND_BATCH_ID.save(deps.storage, &0u128)?;
    UNBOND_BATCHES.save(
        deps.storage,
        0u128,
        &lido_staking_base::state::core::UnbondBatch {
            total_amount: Uint128::zero(),
            expected_amount: Uint128::zero(),
            unbond_items: vec![],
            status: UnbondBatchStatus::New,
            slashing_effect: None,
            unbonded_amount: None,
            withdrawed_amount: None,
        },
    )?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::ExchangeRate {} => to_json_binary(&query_exchange_rate(deps, env)?),
        QueryMsg::UnbondBatch { batch_id } => query_unbond_batch(deps, batch_id),
    }
}

fn query_exchange_rate(_deps: Deps<NeutronQuery>, _env: Env) -> StdResult<Decimal> {
    Decimal::from_str("1.01")
}

fn query_unbond_batch(deps: Deps<NeutronQuery>, batch_id: Uint128) -> StdResult<Binary> {
    to_json_binary(&UNBOND_BATCHES.load(deps.storage, batch_id.into())?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Bond { receiver } => execute_bond(deps, env, info, receiver),
        ExecuteMsg::Unbond {} => execute_unbond(deps, env, info),
        ExecuteMsg::UpdateConfig {
            token_contract,
            puppeteer_contract,
            strategy_contract,
            owner,
            ld_denom,
        } => execute_update_config(
            deps,
            info,
            token_contract,
            puppeteer_contract,
            strategy_contract,
            owner,
            ld_denom,
        ),
        ExecuteMsg::FakeProcessBatch {
            batch_id,
            unbonded_amount,
        } => execute_fake_process_batch(deps, env, info, batch_id, unbonded_amount),
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, env, info, msg),
    }
}

fn execute_fake_process_batch(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    batch_id: Uint128,
    unbonded_amount: Uint128,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "fake_process_batch")];
    let mut unbond_batch = UNBOND_BATCHES.load(deps.storage, batch_id.into())?;
    unbond_batch.unbonded_amount = Some(unbonded_amount);
    unbond_batch.status = UnbondBatchStatus::Unbonded;
    unbond_batch.slashing_effect = Some(
        Decimal::from_str(&unbonded_amount.to_string())?
            / Decimal::from_str(&unbond_batch.expected_amount.to_string())?,
    );
    UNBOND_BATCHES.save(deps.storage, batch_id.into(), &unbond_batch)?;
    attrs.push(attr("batch_id", batch_id.to_string()));
    attrs.push(attr("unbonded_amount", unbonded_amount.to_string()));
    Ok(response("execute-fake_process_batch", CONTRACT_NAME, attrs))
}

fn execute_receive_nft(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "receive_nft")];
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        config.voucher_contract,
        info.sender,
        ContractError::Unauthorized {}
    );
    let voucher: NftInfoResponse<Extension> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.voucher_contract,
            msg: to_json_binary(&lido_staking_base::msg::voucher::QueryMsg::NftInfo {
                token_id: msg.token_id.clone(),
            })?,
        }))?;
    let voucher_extention = voucher.extension.ok_or_else(|| ContractError::InvalidNFT {
        reason: "extension is not set".to_string(),
    })?;

    let batch_id =
        voucher_extention
            .batch_id
            .parse::<u128>()
            .map_err(|_| ContractError::InvalidNFT {
                reason: "invalid batch_id".to_string(),
            })?;

    let mut unbond_batch = UNBOND_BATCHES.load(deps.storage, batch_id)?;
    ensure_eq!(
        unbond_batch.status,
        UnbondBatchStatus::Unbonded,
        ContractError::BatchIsNotUnbonded {}
    );
    let payout_amount = unbond_batch
        .slashing_effect
        .ok_or(ContractError::BatchSlashingEffectIsEmpty {})?
        * voucher_extention.expected_amount;

    unbond_batch.withdrawed_amount =
        Some(unbond_batch.withdrawed_amount.unwrap_or_else(Uint128::zero) + payout_amount);
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: msg.sender,
        amount: vec![Coin {
            denom: config.base_denom,
            amount: payout_amount,
        }],
    });
    UNBOND_BATCHES.save(deps.storage, batch_id, &unbond_batch)?;
    attrs.push(attr("batch_id", batch_id.to_string()));
    attrs.push(attr("payout_amount", payout_amount.to_string()));
    Ok(response("execute-receive_nft", CONTRACT_NAME, attrs).add_message(msg))
}

fn execute_bond(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    let funds = info.funds;
    ensure_ne!(
        funds.len(),
        0,
        ContractError::InvalidFunds {
            reason: "no funds".to_string()
        }
    );
    ensure_eq!(
        funds.len(),
        1,
        ContractError::InvalidFunds {
            reason: "expected 1 denom".to_string()
        }
    );
    let mut attrs = vec![attr("action", "bond")];

    let amount = funds[0].amount;
    let denom = funds[0].denom.to_string();
    check_denom(denom)?;

    let exchange_rate = query_exchange_rate(deps.as_ref(), env)?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));

    let issue_amount = amount * (Decimal::one() / exchange_rate);
    attrs.push(attr("issue_amount", issue_amount.to_string()));

    let receiver = receiver.map_or(Ok::<String, ContractError>(info.sender.to_string()), |a| {
        deps.api.addr_validate(&a)?;
        Ok(a)
    })?;
    attrs.push(attr("receiver", receiver.clone()));

    let msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.token_contract,
        msg: to_json_binary(&TokenExecuteMsg::Mint {
            amount: issue_amount,
            receiver,
        })?,
        funds: vec![],
    })];
    Ok(response("execute-bond", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn check_denom(_denom: String) -> ContractResult<()> {
    //todo: check denom
    Ok(())
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    token_contract: Option<String>,
    puppeteer_contract: Option<String>,
    strategy_contract: Option<String>,
    owner: Option<String>,
    ld_denom: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    ensure_eq!(config.owner, info.sender, ContractError::Unauthorized {});

    let mut attrs = vec![attr("action", "update_config")];
    if let Some(token_contract) = token_contract {
        config.token_contract = token_contract.clone();
        attrs.push(attr("token_contract", token_contract));
    }
    if let Some(puppeteer_contract) = puppeteer_contract {
        config.puppeteer_contract = puppeteer_contract.clone();
        attrs.push(attr("puppeteer_contract", puppeteer_contract));
    }
    if let Some(strategy_contract) = strategy_contract {
        config.strategy_contract = strategy_contract.clone();
        attrs.push(attr("strategy_contract", strategy_contract));
    }
    if let Some(owner) = owner {
        config.owner = owner.clone();
        attrs.push(attr("owner", owner));
    }
    if let Some(ld_denom) = ld_denom {
        config.ld_denom = Some(ld_denom.clone());
        attrs.push(attr("ld_denom", ld_denom));
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("execute-update_config", CONTRACT_NAME, attrs))
}

fn execute_unbond(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let mut attrs = vec![attr("action", "unbond")];
    let unbond_batch_id = UNBOND_BATCH_ID.load(deps.storage)?;
    ensure_eq!(
        info.funds.len(),
        1,
        ContractError::InvalidFunds {
            reason: "Must be one token".to_string(),
        }
    );
    let config = CONFIG.load(deps.storage)?;
    let ld_denom = config.ld_denom.ok_or(ContractError::LDDenomIsNotSet {})?;
    let amount = info.funds[0].amount;
    let denom = info.funds[0].denom.to_string();
    ensure_eq!(
        denom,
        ld_denom,
        ContractError::InvalidFunds {
            reason: "Must be LD token".to_string(),
        }
    );
    let mut unbond_batch = UNBOND_BATCHES.load(deps.storage, unbond_batch_id)?;
    let exchange_rate = query_exchange_rate(deps.as_ref(), env)?;
    attrs.push(attr("exchange_rate", exchange_rate.to_string()));
    let expected_amount = amount * exchange_rate;
    unbond_batch.unbond_items.push(UnbondItem {
        sender: info.sender.to_string(),
        amount,
        expected_amount,
    });
    unbond_batch.total_amount += amount;
    unbond_batch.expected_amount += expected_amount;

    attrs.push(attr("expected_amount", expected_amount.to_string()));
    UNBOND_BATCHES.save(deps.storage, unbond_batch_id, &unbond_batch)?;
    let extension = Some(Metadata {
        description: Some("Withdrawal voucher".into()),
        name: "LDV voucher".to_string(),
        batch_id: unbond_batch_id.to_string(),
        amount,
        expected_amount,
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: "unbond_batch_id".to_string(),
                value: unbond_batch_id.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "received_amount".to_string(),
                value: amount.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "expected_amount".to_string(),
                value: expected_amount.to_string(),
            },
            Trait {
                display_type: None,
                trait_type: "exchange_rate".to_string(),
                value: exchange_rate.to_string(),
            },
        ]),
    });
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.voucher_contract,
        msg: to_json_binary(&VoucherExecuteMsg::Mint {
            owner: info.sender.to_string(),
            token_id: unbond_batch_id.to_string()
                + "_"
                + info.sender.to_string().as_str()
                + "_"
                + &unbond_batch.unbond_items.len().to_string(),
            token_uri: None,
            extension,
        })?,
        funds: vec![],
    });

    Ok(response("execute-unbond", CONTRACT_NAME, attrs).add_message(msg))
}
