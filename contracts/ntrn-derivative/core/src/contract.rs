use crate::{
    error::{ContractError, ContractResult},
    msg::{ExecuteMsg, InstantiateMsg, NftStatus, QueryMsg, ReceiveNftMsg},
    state::{
        Config, BASE_DENOM, CONFIG, CREATE_DENOM_REPLY_ID, DENOM, EXPONENT, SALT, TOKEN_METADATA,
        UNBOND_ID,
    },
};
use cosmos_sdk_proto::cosmos::bank::v1beta1::{DenomUnit, Metadata};
use cosmwasm_std::{
    attr, ensure, ensure_eq, from_json, instantiate2_address, to_json_binary, BankMsg, Binary,
    Coin, CosmosMsg, DenomMetadata, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw721::NftInfoResponse;
use drop_helpers::answer::response;
use drop_staking_base::{
    msg::ntrn_derivative::withdrawal_voucher::{
        ExecuteMsg as WithdrawalVoucherExecuteMsg, Extension as WithdrawalVoucherExtension,
        InstantiateMsg as WithdrawalVoucherInstantiateMsg,
    },
    state::ntrn_derivative::withdrawal_voucher::Metadata as VoucherMetadata,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
    stargate::aux::create_stargate_msg,
};

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
    let create_denom_msg = SubMsg::reply_on_success(
        NeutronMsg::submit_create_denom(&msg.subdenom),
        CREATE_DENOM_REPLY_ID,
    );
    let cosmwasm_std::CodeInfoResponse { checksum, .. } = deps
        .querier
        .query_wasm_code_info(msg.withdrawal_voucher_code_id)?;
    let canonical_self_address = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let canonical_withdrawal_voucher_addr =
        instantiate2_address(&checksum, &canonical_self_address, SALT.as_bytes())?;
    let humanized_withdrawal_voucher_addr =
        deps.api.addr_humanize(&canonical_withdrawal_voucher_addr)?;
    let attrs = vec![
        attr("owner", info.sender.clone()),
        attr("denom", msg.subdenom.clone()),
        attr(
            "withdrawal_voucher_contract",
            humanized_withdrawal_voucher_addr.clone(),
        ),
    ];
    let instantiate_withdrawal_voucher_msg = CosmosMsg::Wasm(WasmMsg::Instantiate2 {
        admin: Some(info.sender.to_string()),
        code_id: msg.withdrawal_voucher_code_id,
        label: get_contract_label("withdrawal-manager"),
        msg: to_json_binary(&WithdrawalVoucherInstantiateMsg {
            name: "Drop NTRN Voucher".to_string(),
            symbol: "DROPV".to_string(),
            minter: env.contract.address.to_string(),
        })?,
        funds: vec![],
        salt: Binary::from(SALT.as_bytes()),
    });
    CONFIG.save(
        deps.storage,
        &Config {
            unbonding_period: msg.unbonding_period,
            withdrawal_voucher: humanized_withdrawal_voucher_addr.clone(),
        },
    )?;
    TOKEN_METADATA.save(deps.storage, &msg.token_metadata)?;
    DENOM.save(deps.storage, &msg.subdenom)?;
    EXPONENT.save(deps.storage, &msg.exponent)?;
    UNBOND_ID.save(deps.storage, &0u64)?;
    Ok(response("instantiate", CONTRACT_NAME, attrs)
        .add_message(instantiate_withdrawal_voucher_msg)
        .add_submessage(create_denom_msg))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            Ok(to_json_binary(&ownership)?)
        }
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            Ok(to_json_binary(&config)?)
        }
        QueryMsg::NftStatus { token_id } => query_nft_status(deps.into_empty(), env, token_id),
    }
}

pub fn query_nft_status(deps: Deps, env: Env, token_id: String) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let nft_info: NftInfoResponse<WithdrawalVoucherExtension> = deps.querier.query_wasm_smart(
        config.withdrawal_voucher.clone(),
        &drop_staking_base::msg::ntrn_derivative::withdrawal_voucher::QueryMsg::NftInfo {
            token_id,
        },
    )?;
    let nft_extension = nft_info
        .extension
        .ok_or_else(|| ContractError::InvalidNFT {
            reason: "extension is not set".to_string(),
        })
        .unwrap();
    let nft_status = match nft_extension.release_at <= env.block.time.seconds() {
        true => NftStatus::Ready {},
        false => NftStatus::NotReady {},
    };
    to_json_binary(&nft_status)
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
        ExecuteMsg::Bond { receiver } => execute_bond(deps, info, receiver),
        ExecuteMsg::Unbond { receiver } => execute_unbond(deps, env, info, receiver),
        ExecuteMsg::ReceiveNft(cw721::Cw721ReceiveMsg {
            sender: _,
            token_id,
            msg: raw_msg,
        }) => {
            let msg: ReceiveNftMsg = from_json(raw_msg)?;
            match msg {
                ReceiveNftMsg::Withdraw { receiver } => {
                    execute_receive_nft_withdraw(deps, env, info, token_id, receiver)
                }
            }
        }
    }
}

fn execute_receive_nft_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_utils::nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    let voucher: NftInfoResponse<WithdrawalVoucherExtension> = deps.querier.query_wasm_smart(
        config.withdrawal_voucher.clone(),
        &drop_staking_base::msg::ntrn_derivative::withdrawal_voucher::QueryMsg::NftInfo {
            token_id,
        },
    )?;
    let voucher_extension = voucher.extension.ok_or_else(|| ContractError::InvalidNFT {
        reason: "extension is not set".to_string(),
    })?;
    let receiver = receiver
        .map(|a| deps.api.addr_validate(&a))
        .unwrap_or_else(|| deps.api.addr_validate(voucher_extension.recipient.as_str()))?
        .to_string();
    let attrs = vec![
        attr("action", "receive_nft"),
        attr("amount", voucher_extension.amount.to_string()),
        attr("to_address", &receiver),
    ];
    let bank_send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: receiver,
        amount: vec![Coin {
            denom: BASE_DENOM.to_string(),
            amount: voucher_extension.amount,
        }],
    });
    ensure_eq!(
        config.withdrawal_voucher,
        info.sender,
        ContractError::Unauthorized {}
    );
    ensure!(
        voucher_extension.release_at <= env.block.time.seconds(),
        ContractError::NftNotReady {}
    );
    Ok(response("execute-receive_nft", CONTRACT_NAME, attrs).add_message(bank_send_msg))
}

fn execute_bond(
    deps: DepsMut,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let amount = cw_utils::must_pay(&info, BASE_DENOM)?;
    let receiver = receiver
        .map(|a| deps.api.addr_validate(&a))
        .unwrap_or_else(|| Ok(info.sender))?;
    let dntrn_denom = DENOM.load(deps.storage)?;
    let attrs = vec![
        attr("action", "bond"),
        attr("amount", amount.to_string()),
        attr("receiver", receiver.clone()),
    ];
    let msg = NeutronMsg::submit_mint_tokens(dntrn_denom, amount, receiver);
    Ok(response("execute-bond", CONTRACT_NAME, attrs).add_message(msg))
}

fn execute_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let dntrn_denom = DENOM.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let amount_to_withdraw = cw_utils::must_pay(&info, &dntrn_denom)?;
    let owner = receiver
        .map(|a| deps.api.addr_validate(&a))
        .unwrap_or_else(|| Ok(info.sender))?;
    let extension = VoucherMetadata {
        description: Some("Withdrawal voucher".into()),
        name: "dNTRN voucher".to_string(),
        amount: amount_to_withdraw,
        release_at: env.block.time.seconds() + config.unbonding_period,
        recipient: owner.to_string(),
    };
    let unbond_id = UNBOND_ID.update(deps.storage, |a| StdResult::Ok(a + 1))?;
    let attrs = vec![
        attr("action", "unbond"),
        attr("amount", amount_to_withdraw.to_string()),
        attr("receiver", owner.clone()),
    ];
    let mint_nft_msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: config.withdrawal_voucher.to_string(),
        msg: to_json_binary(&WithdrawalVoucherExecuteMsg::Mint {
            token_id: unbond_id.to_string(),
            owner: owner.to_string(),
            token_uri: None,
            extension: Some(extension),
        })?,
        funds: vec![],
    });
    let burn_tokens_msg = NeutronMsg::submit_burn_tokens(dntrn_denom, amount_to_withdraw);
    Ok(response("execute-unbond", CONTRACT_NAME, attrs)
        .add_message(mint_nft_msg)
        .add_message(burn_tokens_msg))
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

fn get_contract_label(base: &str) -> String {
    format!("drop-staking-{}", base)
}
