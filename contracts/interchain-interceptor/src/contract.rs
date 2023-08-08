use cosmos_sdk_proto::{
    cosmos::tx::v1beta1::{TxBody, TxRaw},
    traits::Message,
};
use cosmwasm_std::{entry_point, StdError};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{
        msg::NeutronMsg,
        query::{NeutronQuery, QueryRegisteredQueryResponse},
        types::Height,
    },
    interchain_queries::{get_registered_query, v045::new_register_transfers_query_msg},
    interchain_txs::helpers::get_port_id,
    sudo::msg::SudoMsg,
    NeutronError, NeutronResult,
};

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, OpenAckVersion},
    state::{Config, State, CONFIG, STATE},
};

const CONTRACT_NAME: &str = concat!("crates.io:lido-neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const ICA_ID: &str = "LIDO";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            connection_id: msg.connection_id,
            port_id: msg.port_id,
            update_period: msg.update_period,
        },
    )?;
    STATE.save(deps.storage, &State::default())?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterICA {} => execute_register_ica(deps, env),
        ExecuteMsg::RegisterQuery {} => register_transfers_query(deps, env, info),
    }
}

fn execute_register_ica(
    deps: DepsMut<NeutronQuery>,
    env: Env,
) -> NeutronResult<Response<NeutronMsg>> {
    let config: Config = CONFIG.load(deps.storage)?;
    let state: State = STATE.load(deps.storage)?;
    match state.ica {
        None => {
            let register =
                NeutronMsg::register_interchain_account(config.connection_id, ICA_ID.to_string());
            let _key = get_port_id(env.contract.address.as_str(), ICA_ID);

            Ok(Response::new().add_message(register))
        }
        Some(_) => Err(NeutronError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "ICA already registered".to_string(),
        })),
    }
}

fn sudo_open_ack(
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
        STATE.save(
            deps.storage,
            &State {
                last_processed_height: None,
                ica: Some(parsed_version.address),
            },
        )?;
        return Ok(Response::default());
    }
    Err(NeutronError::Std(StdError::GenericErr {
        msg: "can't parse version".to_string(),
    }))
}

pub fn register_transfers_query(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
) -> NeutronResult<Response<NeutronMsg>> {
    let config: Config = CONFIG.load(deps.storage)?;
    let state: State = STATE.load(deps.storage)?;
    if let Some(ica) = state.ica {
        let msg = new_register_transfers_query_msg(
            config.connection_id,
            ica,
            config.update_period,
            None,
        )?;
        Ok(Response::new().add_message(msg))
    } else {
        Err(NeutronError::IntegrationTestsMock {})
    }
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::TxQueryResult {
            query_id,
            height,
            data,
        } => sudo_tx_query_result(deps, env, query_id, height, data),
        // SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, env, query_id),
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
        _ => Ok(Response::default()),
    }
}

fn sudo_tx_query_result(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
    _height: Height,
    data: Binary,
) -> NeutronResult<Response> {
    let _config: Config = CONFIG.load(deps.storage)?;
    let _state: State = STATE.load(deps.storage)?;
    let tx: TxRaw = TxRaw::decode(data.as_slice())?;
    let body: TxBody = TxBody::decode(tx.body_bytes.as_slice())?;
    let registered_query: QueryRegisteredQueryResponse =
        get_registered_query(deps.as_ref(), query_id)?;
    let transactions_filter = registered_query.registered_query.transactions_filter;
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_tx_query_result {:?} filter: {:?}",
            body, transactions_filter
        )
        .as_str(),
    );
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
