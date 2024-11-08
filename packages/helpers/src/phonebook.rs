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
                                .query::<HashMap<String, String>>(&QueryRequest::Wasm(WasmQuery::Smart {
                                    contract_addr: $factory_contract.to_string(),
                                    msg: to_json_binary(&drop_staking_base::msg::factory::QueryMsg::Locate {
                                        contracts: vec![$(stringify!($field_name).to_string()),*],
                                    })?,
                                }))?;

            Phonebook {
                $(
                    $field_name: contracts.get(stringify!($field_name))
                        .unwrap_or_else(|| panic!("Field {} not found in contracts", stringify!($field_name)))
                        .to_string(),
                )*
            }
        }
    };
}
