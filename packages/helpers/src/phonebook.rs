pub static CONTRACTS_CACHE: once_cell::sync::Lazy<
    std::sync::RwLock<std::collections::HashMap<String, String>>,
> = once_cell::sync::Lazy::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

#[macro_export]
macro_rules! get_contracts {
    ($deps:expr, $factory_contract:expr, $($field_name:ident),*) => {
        {
            #[derive(Debug)]
            struct Phonebook {
                $(
                    $field_name: String,
                )*
            }
            use drop_helpers::phonebook::CONTRACTS_CACHE;

             // Collect all requested contract names
            let requested_contracts = vec![$(stringify!($field_name).to_string()),*];
            // List for contracts that need to be queried
            let mut not_cached_contracts = vec![];

            {
                let cache = CONTRACTS_CACHE.read().unwrap();
                for contract_name in &requested_contracts {
                    if !cache.contains_key(contract_name) {
                        not_cached_contracts.push(contract_name.clone());
                    }
                }
            }

            // Query for the missing contracts if needed
            if !not_cached_contracts.is_empty() {
                println!("Querying for missing contracts: {:?}", not_cached_contracts);
                let queried_results:std::collections::HashMap<String, String> = $deps
                    .querier
                    .query(&cosmwasm_std::QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
                        contract_addr: $factory_contract.to_string(),
                        msg: to_json_binary(&drop_staking_base::msg::factory::QueryMsg::Locate {
                            contracts: not_cached_contracts.clone(),
                        })?,
                    }))?;

                // Update the cache with newly queried results
                let mut cache = CONTRACTS_CACHE.write().unwrap();
                for (key, value) in queried_results.iter() {
                    cache.insert(key.clone(), value.clone());
                }
            }

            // Build the Phonebook struct using the updated cache
            let cache = CONTRACTS_CACHE.read().unwrap();
            Phonebook {
                $(
                    $field_name: cache
                        .get(stringify!($field_name))
                        .expect(&format!("{} contract not found in cache", stringify!($field_name)))
                        .to_string(),
                )*
            }
        }
    };
}

pub const CORE_CONTRACT: &str = "core_contract";
pub const WITHDRAWAL_MANAGER_CONTRACT: &str = "withdrawal_manager_contract";
pub const REWARDS_MANAGER_CONTRACT: &str = "rewards_manager_contract";
pub const TOKEN_CONTRACT: &str = "token_contract";
pub const PUPPETEER_CONTRACT: &str = "puppeteer_contract";
pub const WITHDRAWAL_VOUCHER_CONTRACT: &str = "withdrawal_voucher_contract";
pub const STRATEGY_CONTRACT: &str = "strategy_contract";
pub const VALIDATORS_SET_CONTRACT: &str = "validators_set_contract";
pub const DISTRIBUTION_CONTRACT: &str = "distribution_contract";
pub const REWARDS_PUMP_CONTRACT: &str = "rewards_pump_contract";
pub const SPLITTER_CONTRACT: &str = "splitter_contract";
pub const LSM_SHARE_BOND_PROVIDER_CONTRACT: &str = "lsm_share_bond_provider_contract";
pub const NATIVE_BOND_PROVIDER_CONTRACT: &str = "native_bond_provider_contract";
