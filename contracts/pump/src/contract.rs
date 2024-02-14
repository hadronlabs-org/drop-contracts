use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use cosmos_sdk_proto::ibc::applications::transfer::v1::{MsgTransfer, MsgTransferResponse};
use cosmwasm_std::{
    attr, coin, ensure, ensure_eq, entry_point, to_json_binary, Addr, Coin, CosmosMsg, Deps,
    StdError, Uint128,
};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_utils::must_pay;
use lido_helpers::answer::response;
use lido_staking_base::msg::pump::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, OpenAckVersion, QueryMsg, UpdateConfigMsg,
};
use lido_staking_base::state::pump::{Config, CONFIG, ICA, ICA_ID};
use neutron_sdk::bindings::msg::{IbcFee, NeutronMsg};
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::bindings::types::ProtobufAny;
use neutron_sdk::interchain_txs::helpers::decode_message_response;
use neutron_sdk::sudo::msg::{RequestPacket, SudoMsg};
use neutron_sdk::{NeutronError, NeutronResult};
use prost::Message;

use crate::error::{ContractError, ContractResult};

const CONTRACT_NAME: &str = concat!("crates.io:lido-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("contract_name", CONTRACT_NAME),
        attr("contract_version", CONTRACT_VERSION),
        attr("msg", format!("{:?}", msg)),
        attr("sender", &info.sender),
    ];

    CONFIG.save(
        deps.storage,
        &Config {
            dest_address: msg.dest_address.map(Addr::unchecked),
            dest_channel: msg.dest_channel,
            dest_port: msg.dest_port,
            connection_id: msg.connection_id,
            refundee: msg
                .refundee
                .map(|r| deps.api.addr_validate(&r))
                .transpose()?,
            owner: msg
                .owner
                .map(|a| deps.api.addr_validate(&a))
                .transpose()?
                .unwrap_or(info.sender),
            ibc_fees: msg.ibc_fees,
            timeout: msg.timeout,
            local_denom: msg.local_denom,
        },
    )?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Ica {} => query_ica(deps),
    }
}

fn query_config(deps: Deps) -> NeutronResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config).map_err(NeutronError::Std)
}

fn query_ica(deps: Deps) -> NeutronResult<Binary> {
    let ica = ICA.load(deps.storage)?;
    to_json_binary(&ica).map_err(NeutronError::Std)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterICA {} => execute_register_ica(deps, info),
        ExecuteMsg::Push { coins } => execute_push(deps, env, info, coins),
        ExecuteMsg::Refund {} => execute_refund(deps, env),
        ExecuteMsg::UpdateConfig { new_config } => {
            execute_update_config(deps, env, info, *new_config)
        }
    }
}

fn execute_refund(deps: DepsMut<NeutronQuery>, env: Env) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let refundee = config.refundee.ok_or(ContractError::RefundeeIsNotSet {})?;
    let balances = deps.querier.query_all_balances(env.contract.address)?;
    let attrs = vec![attr("action", "refund"), attr("refundee", &refundee)];
    let msg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: refundee.to_string(),
        amount: balances,
    });
    Ok(response("refund", CONTRACT_NAME, attrs).add_message(msg))
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    new_config: UpdateConfigMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, config.owner, ContractError::Unauthorized {});
    let attrs = vec![
        attr("action", "update_config"),
        attr("new_config", format!("{:?}", new_config)),
    ];
    if let Some(dest_address) = new_config.dest_address {
        config.dest_address = Some(Addr::unchecked(dest_address));
    }
    if let Some(dest_channel) = new_config.dest_channel {
        config.dest_channel = Some(dest_channel);
    }
    if let Some(dest_port) = new_config.dest_port {
        config.dest_port = Some(dest_port);
    }
    if let Some(connection_id) = new_config.connection_id {
        config.connection_id = connection_id;
    }
    if let Some(refundee) = new_config.refundee {
        config.refundee = Some(deps.api.addr_validate(&refundee)?);
    }
    if let Some(admin) = new_config.admin {
        config.owner = deps.api.addr_validate(&admin)?;
    }
    if let Some(ibc_fees) = new_config.ibc_fees {
        config.ibc_fees = ibc_fees;
    }
    if let Some(timeout) = new_config.timeout {
        config.timeout = timeout;
    }
    if let Some(local_denom) = new_config.local_denom {
        config.local_denom = local_denom;
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(response("update_config", CONTRACT_NAME, attrs))
}

fn execute_register_ica(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let attrs = vec![
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
    ];
    check_funds(&info, &config, config.ibc_fees.register_fee)?;
    let register_fee: Uint128 = config.ibc_fees.register_fee;
    let register_msg = ICA.register(
        deps.storage,
        config.connection_id,
        ICA_ID,
        coin(register_fee.u128(), config.local_denom),
    )?;
    Ok(response("register-ica", CONTRACT_NAME, attrs).add_message(register_msg))
}

fn execute_push(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    coins: Vec<Coin>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let mut messages = vec![];
    check_funds(
        &info,
        &config,
        config.ibc_fees.ack_fee + config.ibc_fees.recv_fee + config.ibc_fees.timeout_fee,
    )?;
    let attrs = vec![
        attr("action", "push"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
        attr("coins", format!("{:?}", coins)),
    ];
    let fee = IbcFee {
        recv_fee: uint_into_vec_coin(config.ibc_fees.recv_fee, &config.local_denom),
        ack_fee: uint_into_vec_coin(config.ibc_fees.ack_fee, &config.local_denom),
        timeout_fee: uint_into_vec_coin(config.ibc_fees.timeout_fee, &config.local_denom),
    };
    let ica = ICA.get_address(deps.storage)?;
    let timeout_timestamp = env.block.time.plus_seconds(config.timeout.remote).nanos();
    let dst_port = &config
        .dest_port
        .as_ref()
        .ok_or(ContractError::NoDestinationPort {})?;
    let dst_channel = &config
        .dest_channel
        .as_ref()
        .ok_or(ContractError::NoDestinationChannel {})?;
    let dst_address = config
        .dest_address
        .as_ref()
        .ok_or(ContractError::NoDestinationAddress {})?
        .to_string();
    for coin in coins {
        let msg = MsgTransfer {
            source_port: dst_port.to_string(),
            source_channel: dst_channel.to_string(),
            token: Some(ProtoCoin {
                denom: coin.denom,
                amount: coin.amount.to_string(),
            }),
            sender: ica.to_string(),
            receiver: dst_address.to_string(),
            timeout_height: None,
            timeout_timestamp,
        };
        messages.push(compose_msg(
            &config,
            msg,
            &fee,
            "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
            config.timeout.local,
        )?);
    }
    Ok(response("push", CONTRACT_NAME, attrs).add_messages(messages))
}

fn compose_msg<T: prost::Message>(
    config: &Config,
    in_msg: T,
    fee: &IbcFee,
    type_url: String,
    timeout: Option<u64>,
) -> NeutronResult<NeutronMsg> {
    let connection_id = config.connection_id.to_string();
    let mut buf = Vec::new();
    buf.reserve(in_msg.encoded_len());
    if let Err(e) = in_msg.encode(&mut buf) {
        return Err(NeutronError::Std(StdError::generic_err(format!(
            "Encode error: {e}"
        ))));
    }
    let any_msg = ProtobufAny {
        type_url,
        value: Binary::from(buf),
    };
    let cosmos_msg: NeutronMsg = NeutronMsg::submit_tx(
        connection_id,
        ICA_ID.to_string(),
        vec![any_msg],
        "".to_string(),
        timeout.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
        fee.clone(),
    );
    Ok(cosmos_msg)
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    deps.api.debug(&format!(
        "WASMDEBUG: sudo call: {:?} block: {:?}",
        msg, env.block
    ));
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_timeout(deps, env, request),
        SudoMsg::KVQueryResult { .. } | SudoMsg::TxQueryResult { .. } => {
            Err(NeutronError::Std(StdError::GenericErr {
                msg: "KVQueryResult is not supported".to_string(),
            }))
        }
        SudoMsg::OpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => sudo_open_ack(
            deps,
            env,
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        ),
    }
}

pub fn sudo_open_ack(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _port_id: String,
    _channel_id: String,
    _counterparty_channel_id: String,
    counterparty_version: String,
) -> NeutronResult<Response> {
    let parsed_version: Result<OpenAckVersion, _> =
        serde_json_wasm::from_str(counterparty_version.as_str());
    if let Ok(parsed_version) = parsed_version {
        ICA.set_address(deps.storage, parsed_version.address)?;
        Ok(Response::default())
    } else {
        Err(NeutronError::Std(StdError::generic_err(
            "can't parse version",
        )))
    }
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    data: Binary,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let _seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let msg_data: TxMsgData = TxMsgData::decode(data.as_slice())?;
    deps.api
        .debug(&format!("WASMDEBUG: msg_data: data: {msg_data:?}"));

    #[allow(deprecated)]
    for item in msg_data.data {
        match item.msg_type.as_str() {
            "/ibc.applications.transfer.v1.MsgTransferResponse" => {
                let _out: MsgTransferResponse = decode_message_response(&item.data)?;
            }
            _ => {
                deps.api.debug(
                    format!("This type of acknowledgement is not implemented: {item:?}").as_str(),
                );
                return Err(NeutronError::Std(StdError::generic_err(
                    "This type of acknowledgement is not implemented",
                )));
            }
        };
    }
    Ok(response("sudo-response", CONTRACT_NAME, attrs))
}

fn sudo_timeout(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("action", "sudo_timeout"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    ICA.set_timeout(deps.storage)?;
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_timeout: request: {request:?}",
        request = request
    ));
    Ok(response("sudo-timeout", CONTRACT_NAME, attrs))
}

fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    details: String,
) -> NeutronResult<Response> {
    let attrs = vec![
        attr("action", "sudo_error"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
        attr("details", details.clone()),
    ];
    deps.api.debug(&format!(
        "WASMDEBUG: sudo_error: request: {request:?} details: {details:?}",
        request = request,
        details = details
    ));
    let _seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    Ok(response("sudo-error", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

fn uint_into_vec_coin(amount: Uint128, denom: &String) -> Vec<Coin> {
    vec![Coin {
        denom: denom.to_string(),
        amount,
    }]
}

fn check_funds(info: &MessageInfo, config: &Config, amount: Uint128) -> ContractResult<()> {
    let info_amount = must_pay(info, &config.local_denom)?;
    ensure!(
        info_amount >= amount,
        ContractError::InvalidFunds {
            reason: format!(
                "invalid amount: expected at least {}, got {}",
                config.ibc_fees.register_fee, info_amount
            )
        }
    );
    Ok(())
}
