use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        CONFIG, CREATE_DENOM_REPLY_ID, DENOM, EXCHANGE_RATE, EXPONENT, REWARDS_RATE, TOKEN_METADATA,
    },
};
use cosmos_sdk_proto::cosmos::bank::v1beta1::{DenomUnit, Metadata};
use cosmwasm_std::{
    attr, entry_point, to_json_binary, Attribute, BankMsg, Binary, Coin, CosmosMsg, Decimal,
    DenomMetadata, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg, Uint128,
};
use drop_helpers::answer::response;
use drop_staking_base::{
    msg::{core::QueryMsg as CoreQueryMsg, factory::QueryMsg as FactoryQueryMsg},
    state::factory::State as FactoryState,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
    stargate::aux::create_stargate_msg,
};

const CONTRACT_NAME: &str = concat!("crates.io:drop-helper__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    msg.config.validate_base_denom(deps.as_ref())?;
    msg.config.validate_splitting_targets(deps.as_ref())?;
    msg.config.validate_factory_addr(deps.as_ref())?;

    CONFIG.save(deps.storage, &msg.config)?;
    EXCHANGE_RATE.save(deps.storage, &Decimal::one())?;
    REWARDS_RATE.save(deps.storage, &Decimal::zero())?;
    DENOM.save(deps.storage, &msg.subdenom)?;
    EXPONENT.save(deps.storage, &msg.exponent)?;
    TOKEN_METADATA.save(deps.storage, &msg.token_metadata)?;

    let create_denom_submsg = SubMsg::reply_on_success(
        NeutronMsg::submit_create_denom(msg.subdenom),
        CREATE_DENOM_REPLY_ID,
    );
    Ok(
        response("instantiate", CONTRACT_NAME, Vec::<Attribute>::new())
            .add_submessage(create_denom_submsg),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(
            &cw_ownable::get_ownership(deps.storage)?.owner,
        )?),
        QueryMsg::ExchangeRate {} => Ok(to_json_binary(&query_exchange_rate(deps, env)?)?),
        QueryMsg::Rewards {} => Ok(to_json_binary(&query_rewards(deps, env)?)?),
    }
}

fn query_rewards(deps: Deps, env: Env) -> StdResult<Uint128> {
    let base_denom = CONFIG.load(deps.storage)?.base_denom;
    let dasset_contract_balance = deps
        .querier
        .query_balance(env.contract.address, base_denom.clone())?;
    let core_exchange_rate = query_core_exchange_rate(deps)?;
    let asset_contract_balance = core_exchange_rate.checked_mul(Decimal::from_ratio(
        dasset_contract_balance.amount,
        Uint128::one(),
    ))?;
    let lazy_denom: String = DENOM.load(deps.storage)?;
    let lazy_total_supply = deps.querier.query_supply(lazy_denom)?;
    let backing_excess_asset = lazy_total_supply
        .amount
        .abs_diff(asset_contract_balance.to_uint_floor());
    let backing_excess_dasset =
        Decimal::from_ratio(backing_excess_asset, Uint128::one()) / core_exchange_rate;
    Ok(backing_excess_dasset.to_uint_floor())
}

fn query_exchange_rate(deps: Deps, env: Env) -> StdResult<Decimal> {
    let base_denom = CONFIG.load(deps.storage)?.base_denom;
    let dasset_contract_balance = deps
        .querier
        .query_balance(env.contract.address, base_denom.clone())?;
    let core_exchange_rate = query_core_exchange_rate(deps)?;
    let asset_contract_balance = core_exchange_rate.checked_mul(Decimal::from_ratio(
        dasset_contract_balance.amount,
        Uint128::one(),
    ))?;
    let lazy_denom: String = DENOM.load(deps.storage)?;
    let lazy_total_supply = deps.querier.query_supply(lazy_denom)?;
    let exchange_rate = asset_contract_balance
        .checked_div(Decimal::from_ratio(
            lazy_total_supply.amount,
            Uint128::one(),
        ))
        .unwrap();
    Ok(exchange_rate)
}

fn query_core_exchange_rate(deps: Deps) -> StdResult<Decimal> {
    let config = CONFIG.load(deps.storage)?;
    let factory_addr = config.factory_addr;
    let factory_state: FactoryState = deps
        .querier
        .query_wasm_smart(factory_addr, &FactoryQueryMsg::State {})?;
    let core_addr = factory_state.core_contract;
    let core_exchange_rate: Decimal = deps
        .querier
        .query_wasm_smart(core_addr, &CoreQueryMsg::ExchangeRate {})?;

    Ok(core_exchange_rate)
}

#[cfg_attr(not(feature = "library"), entry_point)]
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
        ExecuteMsg::Bond => execute_bond(deps, info),
        ExecuteMsg::Unbond => execute_unbond(deps, info),
        ExecuteMsg::WithdrawRewards => execute_withdaw_rewards(deps, env, info),
    }
}

fn execute_withdaw_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let rewards = query_rewards(deps.as_ref(), env)?;
    let attrs = Vec::<Attribute>::new();
    let splitting_total_weight = config
        .splitting_targets
        .clone()
        .into_iter()
        .fold(Uint128::zero(), |accum, target| {
            accum + target.unbonding_weight
        });
    let mut msgs = Vec::<CosmosMsg<NeutronMsg>>::new();
    for splitting_target in config.splitting_targets.into_iter() {
        let numerator = Decimal::from_ratio(splitting_target.unbonding_weight, Uint128::one());
        let denominator = Decimal::from_ratio(splitting_total_weight, Uint128::one());
        let rewards_part = rewards * (numerator / denominator);
        let bank_send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.base_denom.clone(),
                amount: rewards_part,
            }],
        });
        msgs.push(bank_send_msg);
    }
    Ok(response("execute-withdraw-rewards", CONTRACT_NAME, attrs).add_messages(msgs))
}

fn execute_unbond(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let lazy_denom = DENOM.load(deps.storage)?;
    let sent_amount = cw_utils::must_pay(&info, &lazy_denom)?;
    let core_exchange_rate = query_core_exchange_rate(deps.as_ref())?;
    let return_amount = Decimal::from_ratio(sent_amount, Uint128::one()) / core_exchange_rate;
    let floor_return_amount = return_amount.to_uint_floor();

    let attrs = vec![
        attr("sent_amount", sent_amount.to_string()),
        attr("floor_return_amount", floor_return_amount.to_string()),
        attr("receiver", info.sender.clone()),
    ];
    let burn_msg = NeutronMsg::submit_burn_tokens(lazy_denom, sent_amount);
    let bank_send_msg: CosmosMsg<NeutronMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: config.base_denom,
            amount: floor_return_amount,
        }],
    });
    Ok(response("execute-unbond", CONTRACT_NAME, attrs)
        .add_message(burn_msg)
        .add_message(bank_send_msg))
}

fn execute_bond(deps: DepsMut, info: MessageInfo) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let sent_amount = cw_utils::must_pay(&info, &config.base_denom)?;
    let core_exchange_rate = query_core_exchange_rate(deps.as_ref())?;
    let issue_amount = sent_amount * core_exchange_rate;
    let lazy_denom = DENOM.load(deps.storage)?;

    let attrs = vec![
        attr("sent_amount", sent_amount.to_string()),
        attr("core_exchange_rate", core_exchange_rate.to_string()),
        attr("issue_amount", issue_amount.to_string()),
        attr("receiver", info.sender.clone()),
    ];
    let msg = NeutronMsg::submit_mint_tokens(lazy_denom, issue_amount, info.sender);
    Ok(response("execute-bond", CONTRACT_NAME, attrs).add_message(msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.id {
        CREATE_DENOM_REPLY_ID => {
            let subdenom = DENOM.load(deps.storage)?;
            let full_denom = query_full_denom(deps.as_ref(), &env.contract.address, subdenom)?;
            let token_metadata = TOKEN_METADATA.load(deps.storage)?;
            let exponent = EXPONENT.load(deps.storage)?;
            let msg = create_set_denom_metadata_msg(
                env.contract.address.into_string(),
                full_denom.denom.clone(),
                token_metadata.clone(),
                exponent,
            );
            deps.api
                .debug(&format!("WASMDEBUG: msg: {:?}", token_metadata));
            DENOM.save(deps.storage, &full_denom.denom)?;
            TOKEN_METADATA.remove(deps.storage);
            Ok(Response::new()
                .add_attribute("full_denom", full_denom.denom)
                .add_message(msg))
        }
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn create_set_denom_metadata_msg(
    contract_address: String,
    denom: String,
    token_metadata: DenomMetadata,
    exponent: u32,
) -> CosmosMsg<NeutronMsg> {
    create_stargate_msg(
        "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata",
        neutron_sdk::proto_types::osmosis::tokenfactory::v1beta1::MsgSetDenomMetadata {
            sender: contract_address,
            metadata: Some(Metadata {
                denom_units: vec![
                    DenomUnit {
                        denom: denom.clone(),
                        exponent: 0,
                        aliases: vec![],
                    },
                    DenomUnit {
                        denom: token_metadata.display.clone(),
                        exponent,
                        aliases: vec![],
                    },
                ],
                base: denom,
                display: token_metadata.display,
                name: token_metadata.name,
                description: token_metadata.description,
                symbol: token_metadata.symbol,
                uri: token_metadata.uri,
                uri_hash: token_metadata.uri_hash,
            }),
        },
    )
}
