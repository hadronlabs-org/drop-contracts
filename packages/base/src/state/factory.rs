use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use std::hash::Hash;

#[cw_serde]
pub struct CodeIds {
    pub token_code_id: u64,
    pub core_code_id: u64,
    pub puppeteer_code_id: u64,
    pub withdrawal_voucher_code_id: u64,
    pub withdrawal_manager_code_id: u64,
    pub strategy_code_id: u64,
    pub validators_set_code_id: u64,
    pub distribution_code_id: u64,
    pub rewards_manager_code_id: u64,
    pub splitter_code_id: u64,
    pub rewards_pump_code_id: u64,
    pub lsm_share_bond_provider_code_id: u64,
    pub native_bond_provider_code_id: u64,
}

#[cw_serde]
pub struct RemoteOpts {
    pub denom: String,
    pub update_period: u64, // ICQ
    pub connection_id: String,
    pub port_id: String,
    pub transfer_channel_id: String,
    pub reverse_transfer_channel_id: String,
    pub timeout: Timeout,
}

#[cw_serde]
pub struct Timeout {
    pub local: u64,
    pub remote: u64,
}

#[cw_serde]
pub struct PauseInfoResponse {
    pub withdrawal_manager: drop_helpers::pause::PauseInfoResponse,
    pub core: crate::state::core::Pause,
    pub rewards_manager: drop_helpers::pause::PauseInfoResponse,
}

#[cw_serde]
pub struct Phonebook {
    pub map: std::collections::HashMap<String, Addr>,
}

pub const STATE: Item<Phonebook> = Item::new("state");

impl Phonebook {
    pub fn get_as_result<'a>(
        &'a self,
        key: &'a str,
    ) -> Result<&Addr, crate::error::factory::ContractError> {
        self.map.get(key).ok_or(
            crate::error::factory::ContractError::ContractAddressNotFound {
                name: key.to_string(),
            },
        )
    }

    pub fn new<K, const N: usize>(arr: [(K, Addr); N]) -> Self
    where
        // Bounds from impl:
        K: Eq + Hash + Into<String> + Clone + std::fmt::Display,
    {
        let map = arr
            .iter()
            .clone()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        Self { map }
    }
}
