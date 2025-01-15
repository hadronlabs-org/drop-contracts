use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

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

pub const STATE: Map<&str, Addr> = Map::new("state");
