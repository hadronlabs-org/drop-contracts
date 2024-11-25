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
            let contracts = $deps
                                .querier
                                .query::<std::collections::HashMap<String, String>>(&cosmwasm_std::QueryRequest::Wasm(cosmwasm_std::WasmQuery::Smart {
                                    contract_addr: $factory_contract.to_string(),
                                    msg: to_json_binary(&drop_staking_base::msg::factory::QueryMsg::Locate {
                                        contracts: vec![$(stringify!($field_name).to_string()),*],
                                    })?,
                                }))?;

            Phonebook {
                $(
                    $field_name: contracts.get(stringify!($field_name))
                        .expect(&format!("{} contract not found", stringify!($field_name)))
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
