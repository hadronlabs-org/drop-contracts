use crate::error::{ContractError, ContractResult};
use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use cosmos_sdk_proto::ibc::applications::transfer::v1::{MsgTransfer, MsgTransferResponse};
use cosmwasm_std::{attr, to_json_binary, Addr, Coin, CosmosMsg, Deps, StdError};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response};
use drop_helpers::answer::response;
use drop_helpers::ibc_fee::query_ibc_fee;
use drop_staking_base::msg::pump::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, OpenAckVersion, QueryMsg, UpdateConfigMsg,
};
use drop_staking_base::state::pump::{Config, CONFIG, ICA, ICA_ID};
use neutron_sdk::bindings::msg::{IbcFee, NeutronMsg};
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::bindings::types::ProtobufAny;
use neutron_sdk::interchain_txs::helpers::decode_message_response;
use neutron_sdk::sudo::msg::{RequestPacket, SudoMsg};
use neutron_sdk::NeutronError;
use prost::Message;

const CONTRACT_NAME: &str = concat!("crates.io:drop-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let attrs = vec![
        attr("contract_name", CONTRACT_NAME),
        attr("contract_version", CONTRACT_VERSION),
        attr("msg", format!("{:?}", msg)),
        attr("sender", &info.sender),
    ];
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.owner.unwrap_or(info.sender.to_string()).as_str()),
    )?;
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
            timeout: msg.timeout,
            local_denom: msg.local_denom,
        },
    )?;
    Ok(response("instantiate", CONTRACT_NAME, attrs))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Ica {} => query_ica(deps),
        QueryMsg::Ownership {} => {
            let ownership = cw_ownable::get_ownership(deps.storage)?;
            Ok(to_json_binary(&ownership)?)
        }
    }
}

fn query_config(deps: Deps) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_ica(deps: Deps) -> ContractResult<Binary> {
    let ica = ICA.load(deps.storage)?;
    Ok(to_json_binary(&ica)?)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterICA {} => execute_register_ica(deps, info),
        ExecuteMsg::Push { coins } => execute_push(deps, env, coins),
        ExecuteMsg::Refund { coins } => execute_refund(deps, info, coins),
        ExecuteMsg::UpdateConfig { new_config } => {
            execute_update_config(deps, env, info, *new_config)
        }
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

fn execute_refund(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    coins: Vec<Coin>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let refundee = config.refundee.ok_or(ContractError::RefundeeIsNotSet {})?;

    // only allow either owner or refundee to execute this method
    if cw_ownable::assert_owner(deps.storage, &info.sender).is_err() && info.sender != refundee {
        return Err(ContractError::Unauthorized {});
    }

    let attrs = vec![attr("action", "refund"), attr("refundee", &refundee)];
    let msg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: refundee.into_string(),
        amount: coins,
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
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
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
        attr("action", "register_ica"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
    ];
    let register_fee = info
        .funds
        .into_iter()
        .find(|f| f.denom == config.local_denom)
        .ok_or(ContractError::InvalidFunds {
            reason: format!("missing fee in denom {}", config.local_denom),
        })?;
    let register_msg = ICA.register(deps.storage, config.connection_id, ICA_ID, register_fee)?;
    Ok(response("register-ica", CONTRACT_NAME, attrs).add_message(register_msg))
}

fn execute_push(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    coins: Vec<Coin>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let mut messages = vec![];
    let attrs = vec![
        attr("action", "push"),
        attr("connection_id", &config.connection_id),
        attr("ica_id", ICA_ID),
        attr("coins", format!("{:?}", coins)),
    ];
    let fee = query_ibc_fee(deps.as_ref(), &config.local_denom)?;
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
) -> ContractResult<NeutronMsg> {
    let connection_id = config.connection_id.to_string();
    let mut buf = Vec::with_capacity(in_msg.encoded_len());
    in_msg
        .encode(&mut buf)
        .map_err(|e| StdError::generic_err(format!("Encode error: {e}")))?;
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

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> ContractResult<Response> {
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_timeout(deps, env, request),
        SudoMsg::KVQueryResult { .. } | SudoMsg::TxQueryResult { .. } => {
            Err(StdError::generic_err("KVQueryResult and TxQueryResult are not supported").into())
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
    port_id: String,
    channel_id: String,
    _counterparty_channel_id: String,
    counterparty_version: String,
) -> ContractResult<Response> {
    let parsed_version: Result<OpenAckVersion, _> =
        serde_json_wasm::from_str(counterparty_version.as_str());
    if let Ok(parsed_version) = parsed_version {
        ICA.set_address(deps.storage, parsed_version.address, port_id, channel_id)?;
        Ok(Response::default())
    } else {
        Err(StdError::generic_err("can't parse version").into())
    }
}

fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    data: Binary,
) -> ContractResult<Response> {
    let attrs = vec![
        attr("action", "sudo_response"),
        attr("request_id", request.sequence.unwrap_or(0).to_string()),
    ];
    let _seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let msg_data: TxMsgData = TxMsgData::decode(data.as_slice()).map_err(NeutronError::from)?;
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
                return Err(StdError::generic_err(
                    "This type of acknowledgement is not implemented",
                )
                .into());
            }
        };
    }
    Ok(response("sudo-response", CONTRACT_NAME, attrs))
}

fn sudo_timeout(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
) -> ContractResult<Response> {
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
) -> ContractResult<Response> {
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
