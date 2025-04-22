#![cfg(not(target_arch = "wasm32"))]

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, to_json_binary, AllBalanceResponse, Api, BalanceResponse, BankQuery, Binary, Coin,
    ContractResult, CustomQuery, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, Uint128,
};

use neutron_sdk::bindings::query::NeutronQuery;

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

#[cw_serde]
pub struct CustomQueryWrapper {}

// implement custom query
impl CustomQuery for CustomQueryWrapper {}

#[derive(Clone)]
pub struct CustomMockApi {}

impl Api for CustomMockApi {
    fn addr_validate(&self, input: &str) -> cosmwasm_std::StdResult<cosmwasm_std::Addr> {
        Ok(cosmwasm_std::Addr::unchecked(input))
    }

    fn addr_canonicalize(
        &self,
        _input: &str,
    ) -> cosmwasm_std::StdResult<cosmwasm_std::CanonicalAddr> {
        Ok(cosmwasm_std::CanonicalAddr::from(
            "some_canonical_address".as_bytes(),
        ))
    }

    fn addr_humanize(
        &self,
        _canonical: &cosmwasm_std::CanonicalAddr,
    ) -> cosmwasm_std::StdResult<cosmwasm_std::Addr> {
        Ok(cosmwasm_std::Addr::unchecked(
            "some_humanized_address".to_string(),
        ))
    }

    fn secp256k1_verify(
        &self,
        _message_hash: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> Result<bool, cosmwasm_std::VerificationError> {
        Ok(true)
    }

    fn secp256k1_recover_pubkey(
        &self,
        _message_hash: &[u8],
        _signature: &[u8],
        _recovery_param: u8,
    ) -> Result<Vec<u8>, cosmwasm_std::RecoverPubkeyError> {
        Ok(vec![])
    }

    fn ed25519_verify(
        &self,
        _message: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> Result<bool, cosmwasm_std::VerificationError> {
        Ok(true)
    }

    fn ed25519_batch_verify(
        &self,
        _messages: &[&[u8]],
        _signatures: &[&[u8]],
        _public_keys: &[&[u8]],
    ) -> Result<bool, cosmwasm_std::VerificationError> {
        Ok(true)
    }

    fn debug(&self, message: &str) {
        println!("{message}");
    }
}

pub fn mock_dependencies_with_api(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, CustomMockApi, WasmMockQuerier, NeutronQuery> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: CustomMockApi {},
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

type WasmFn = dyn Fn(&Binary) -> ContractResult<Binary>;
type CustomFn = dyn Fn(&QueryRequest<NeutronQuery>) -> Binary;

pub struct WasmMockQuerier {
    base: MockQuerier<NeutronQuery>,
    bank_query_responses: HashMap<String, Binary>,
    query_responses: HashMap<u64, Binary>,
    registered_queries: HashMap<u64, Binary>,
    ibc_query_responses: HashMap<String, Binary>,
    wasm_query_responses: RefCell<HashMap<String, Vec<Box<WasmFn>>>>, // fml
    custom_query_responses: RefCell<Vec<Box<CustomFn>>>,              // fml
    stargate_query_responses: RefCell<HashMap<String, Vec<Box<WasmFn>>>>, // fml
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<NeutronQuery> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<NeutronQuery>) -> QuerierResult {
        match &request {
            QueryRequest::Bank(bank_query) => match bank_query {
                BankQuery::Balance { address, .. } => {
                    let custom_balance = self.bank_query_responses.get(address);

                    if let Some(balance) = custom_balance {
                        SystemResult::Ok(ContractResult::Ok(balance.clone()))
                    } else {
                        self.base.handle_query(request)
                    }
                }
                BankQuery::AllBalances { address, .. } => {
                    let custom_balance = self.bank_query_responses.get(address);

                    if let Some(balances) = custom_balance {
                        SystemResult::Ok(ContractResult::Ok(balances.clone()))
                    } else {
                        self.base.handle_query(request)
                    }
                }
                _ => self.base.handle_query(request),
            },
            QueryRequest::Ibc(cosmwasm_std::IbcQuery::Channel {
                channel_id,
                port_id,
            }) => {
                let mut channel_port: String = (*channel_id).clone();
                if let Some(port_id) = (*port_id).clone() {
                    channel_port.push('/');
                    channel_port.push_str(&port_id);
                } else {
                    channel_port.push_str("/*");
                }
                SystemResult::Ok(
                    ContractResult::Ok(
                        (*self.ibc_query_responses.get(&channel_port).unwrap_or(
                            &to_json_binary(&cosmwasm_std::ChannelResponse { channel: None })
                                .unwrap(),
                        ))
                        .clone(),
                    )
                    .clone(),
                )
            }
            QueryRequest::Stargate { path, data } => {
                let mut stargate_query_responses = self.stargate_query_responses.borrow_mut();
                let responses = match stargate_query_responses.get_mut(path) {
                    None => Err(SystemError::UnsupportedRequest {
                        kind: format!(
                            "Stargate query is not mocked. Path: {} Data {}",
                            path,
                            String::from_utf8(data.0.clone()).unwrap()
                        ),
                    }),
                    Some(responses) => Ok(responses),
                }
                .unwrap();
                if responses.is_empty() {
                    return SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: format!(
                            "Stargate query is not mocked. Path: {} Data {}",
                            path,
                            String::from_utf8(data.0.clone()).unwrap()
                        ),
                    });
                }
                let response = responses.remove(0);
                SystemResult::Ok(response(data))
            }
            QueryRequest::Custom(custom_query) => match custom_query {
                NeutronQuery::InterchainQueryResult { query_id } => SystemResult::Ok(
                    ContractResult::Ok((*self.query_responses.get(query_id).unwrap()).clone()),
                ),
                NeutronQuery::RegisteredInterchainQuery { query_id } => SystemResult::Ok(
                    ContractResult::Ok((*self.registered_queries.get(query_id).unwrap()).clone()),
                ),
                NeutronQuery::RegisteredInterchainQueries {
                    owners: _owners,
                    connection_id: _connection_id,
                    pagination: _pagination,
                } => {
                    todo!()
                }
                NeutronQuery::InterchainAccountAddress { .. } => {
                    todo!()
                }
                _ => {
                    let mut custom_query_responses = self.custom_query_responses.borrow_mut();
                    if custom_query_responses.len() == 0 {
                        return SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: format!("Custom query is not mocked: {:?}", custom_query),
                        });
                    }
                    let response = custom_query_responses.remove(0);
                    SystemResult::Ok(ContractResult::Ok(response(request)))
                }
            },
            QueryRequest::Wasm(wasm_query) => match wasm_query {
                cosmwasm_std::WasmQuery::Smart { contract_addr, msg } => {
                    let mut wasm_query_responses = self.wasm_query_responses.borrow_mut();
                    let responses = match wasm_query_responses.get_mut(contract_addr) {
                        None => Err(SystemError::UnsupportedRequest {
                            kind: format!(
                                "Wasm contract {} query is not mocked. Query {}",
                                contract_addr,
                                String::from_utf8(msg.0.clone()).unwrap()
                            ),
                        }),
                        Some(responses) => Ok(responses),
                    }
                    .unwrap();
                    if responses.is_empty() {
                        return SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: format!(
                                "Wasm contract {} query is not mocked. Query {}",
                                contract_addr,
                                String::from_utf8(msg.0.clone()).unwrap()
                            ),
                        });
                    }
                    let response = responses.remove(0);
                    SystemResult::Ok(response(msg))
                }
                cosmwasm_std::WasmQuery::CodeInfo { code_id } => {
                    let mut stargate_query_responses = self.stargate_query_responses.borrow_mut();
                    let responses = match stargate_query_responses
                        .get_mut("/cosmos.wasm.v1.Query/QueryCodeRequest")
                    {
                        None => Err(SystemError::UnsupportedRequest {
                            kind: format!(
                                "Stargate query is not mocked. Path: {} Data {}",
                                "/cosmos.wasm.v1.Query/QueryCodeRequest", code_id
                            ),
                        }),
                        Some(responses) => Ok(responses),
                    }
                    .unwrap();
                    if responses.is_empty() {
                        return SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "No such mocked queries found".to_string(),
                        });
                    }
                    SystemResult::Ok(responses[0](&to_json_binary(&code_id).unwrap()))
                }
                cosmwasm_std::WasmQuery::ContractInfo { contract_addr } => {
                    let mut wasm_responses = self.wasm_query_responses.borrow_mut();
                    let responses = match wasm_responses.get_mut(contract_addr) {
                        None => Err(SystemError::UnsupportedRequest {
                            kind: format!(
                                "Wasm contract {} contract info query is not mocked. Query",
                                contract_addr
                            ),
                        }),
                        Some(responses) => Ok(responses),
                    }
                    .unwrap();
                    if responses.is_empty() {
                        return SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "No such mocked contract info queries found".to_string(),
                        });
                    }
                    SystemResult::Ok(responses[0](&to_json_binary(&contract_addr).unwrap()))
                }
                cosmwasm_std::WasmQuery::Raw { contract_addr, key } => {
                    let mut wasm_responses = self.wasm_query_responses.borrow_mut();
                    let responses = match wasm_responses.get_mut(contract_addr) {
                        None => Err(SystemError::UnsupportedRequest {
                            kind: format!(
                                "Wasm contract {} raw query is not mocked. Raw query {}",
                                contract_addr,
                                hex::encode(key)
                            ),
                        }),
                        Some(responses) => Ok(responses),
                    }
                    .unwrap();
                    if responses.is_empty() {
                        return SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "No such mocked raw queries found".to_string(),
                        });
                    }
                    let response = responses.remove(0);
                    SystemResult::Ok(response(key))
                }
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "Unsupported wasm request given".to_string(),
                }),
            },
            _ => self.base.handle_query(request),
        }
    }

    pub fn add_bank_query_response(&mut self, address: String, response: BalanceResponse) {
        self.bank_query_responses
            .insert(address, to_json_binary(&response).unwrap());
    }
    pub fn add_all_balances_query_response(
        &mut self,
        address: String,
        response: AllBalanceResponse,
    ) {
        self.bank_query_responses
            .insert(address, to_json_binary(&response).unwrap());
    }
    pub fn add_query_response(&mut self, query_id: u64, response: Binary) {
        self.query_responses.insert(query_id, response);
    }
    pub fn add_ibc_channel_response(
        &mut self,
        channel_id: Option<String>,
        port_id: Option<String>,
        response: cosmwasm_std::ChannelResponse,
    ) {
        // channel-0/transfer
        // */transfer
        // channel-0/*
        // */*
        let mut channel_port: String;
        if let Some(channel_id) = channel_id {
            channel_port = channel_id.clone() + "/";
        } else {
            channel_port = "*/".to_string();
        }
        if let Some(port_id) = port_id {
            channel_port.push_str(port_id.clone().as_str());
        } else {
            channel_port.push('*');
        }
        self.ibc_query_responses
            .insert(channel_port, to_json_binary(&response).unwrap());
    }
    pub fn add_registered_queries(&mut self, query_id: u64, response: Binary) {
        self.registered_queries.insert(query_id, response);
    }
    pub fn add_wasm_query_response<F>(&mut self, contract_address: &str, response_func: F)
    where
        F: 'static + Fn(&Binary) -> ContractResult<Binary>,
    {
        let mut wasm_responses = self.wasm_query_responses.borrow_mut();
        let response_funcs = wasm_responses
            .entry(contract_address.to_string())
            .or_default();

        response_funcs.push(Box::new(response_func));
    }
    pub fn add_custom_query_response<F>(&mut self, response_func: F)
    where
        F: 'static + Fn(&QueryRequest<NeutronQuery>) -> Binary,
    {
        let mut custom_query_responses = self.custom_query_responses.borrow_mut();
        custom_query_responses.push(Box::new(response_func));
    }
    pub fn add_stargate_query_response<F>(&mut self, path: &str, response_func: F)
    where
        F: 'static + Fn(&Binary) -> ContractResult<Binary>,
    {
        let mut stargate_responses = self.stargate_query_responses.borrow_mut();
        let response_funcs = stargate_responses.entry(path.to_string()).or_default();
        response_funcs.push(Box::new(response_func));
    }
}

#[derive(Clone, Default)]
pub struct BalanceQuerier {
    _balances: HashMap<String, Coin>,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    _balances: HashMap<String, HashMap<String, Uint128>>,
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<NeutronQuery>) -> Self {
        WasmMockQuerier {
            base,
            bank_query_responses: HashMap::new(),
            query_responses: HashMap::new(),
            registered_queries: HashMap::new(),
            ibc_query_responses: HashMap::new(),
            wasm_query_responses: HashMap::new().into(),
            stargate_query_responses: HashMap::new().into(),
            custom_query_responses: Vec::new().into(),
        }
    }
}

pub fn mock_state_query(deps: &mut OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery>) {
    deps.querier
        .add_wasm_query_response("factory_contract", |_| {
            let contracts = HashMap::from([
                ("core_contract".to_string(), "core_contract".to_string()),
                ("token_contract".to_string(), "token_contract".to_string()),
                (
                    "withdrawal_voucher_contract".to_string(),
                    "withdrawal_voucher_contract".to_string(),
                ),
                (
                    "withdrawal_manager_contract".to_string(),
                    "withdrawal_manager_contract".to_string(),
                ),
                (
                    "strategy_contract".to_string(),
                    "strategy_contract".to_string(),
                ),
                (
                    "validators_set_contract".to_string(),
                    "validators_set_contract".to_string(),
                ),
                (
                    "distribution_contract".to_string(),
                    "distribution_contract".to_string(),
                ),
                (
                    "puppeteer_contract".to_string(),
                    "puppeteer_contract".to_string(),
                ),
                (
                    "rewards_manager_contract".to_string(),
                    "rewards_manager_contract".to_string(),
                ),
                (
                    "rewards_pump_contract".to_string(),
                    "rewards_pump_contract".to_string(),
                ),
                (
                    "splitter_contract".to_string(),
                    "splitter_contract".to_string(),
                ),
                (
                    "lsm_share_bond_provider_contract".to_string(),
                    "lsm_share_bond_provider_contract".to_string(),
                ),
                (
                    "native_bond_provider_contract".to_string(),
                    "native_bond_provider_contract".to_string(),
                ),
            ]);
            cosmwasm_std::ContractResult::Ok(to_json_binary(&contracts).unwrap())
        });
}
