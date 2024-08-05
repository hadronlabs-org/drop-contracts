#![cfg(not(target_arch = "wasm32"))]

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, to_json_binary, Api, Binary, Coin, ContractResult, CustomQuery, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128,
};

use neutron_sdk::bindings::query::NeutronQuery;

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

#[cw_serde]
pub struct CustomQueryWrapper {}

// implement custom query
impl CustomQuery for CustomQueryWrapper {}

#[derive(Clone)]
pub struct MockApi {}

impl Api for MockApi {
    fn addr_validate(&self, _input: &str) -> cosmwasm_std::StdResult<cosmwasm_std::Addr> {
        Ok(cosmwasm_std::Addr::unchecked("some_address".to_string()))
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

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, NeutronQuery> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi {},
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

type WasmFn = dyn Fn(&Binary) -> Binary;
type CustomFn = dyn Fn(&QueryRequest<NeutronQuery>) -> Binary;

pub struct WasmMockQuerier {
    base: MockQuerier<NeutronQuery>,
    query_responses: HashMap<u64, Binary>,
    registered_queries: HashMap<u64, Binary>,
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
                SystemResult::Ok(ContractResult::Ok(response(data)))
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
                    SystemResult::Ok(ContractResult::Ok(response(msg)))
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
                    SystemResult::Ok(ContractResult::Ok(responses[0](
                        &to_json_binary(&code_id).unwrap(),
                    )))
                }
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: format!("Unsupported wasm request given"),
                }),
            },
            _ => self.base.handle_query(request),
        }
    }

    pub fn add_query_response(&mut self, query_id: u64, response: Binary) {
        self.query_responses.insert(query_id, response);
    }
    pub fn add_registered_queries(&mut self, query_id: u64, response: Binary) {
        self.registered_queries.insert(query_id, response);
    }
    pub fn add_wasm_query_response<F>(&mut self, contract_address: &str, response_func: F)
    where
        F: 'static + Fn(&Binary) -> Binary,
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
        F: 'static + Fn(&Binary) -> Binary,
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
            query_responses: HashMap::new(),
            registered_queries: HashMap::new(),
            wasm_query_responses: HashMap::new().into(),
            stargate_query_responses: HashMap::new().into(),
            custom_query_responses: Vec::new().into(),
        }
    }
}
