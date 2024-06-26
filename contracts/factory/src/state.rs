use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;
use drop_staking_base::msg::token::DenomMetadata;

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
}

#[cw_serde]
pub struct RemoteOpts {
    pub denom: String,
    pub update_period: u64, // ICQ
    pub connection_id: String,
    pub port_id: String,
    pub transfer_channel_id: String,
}

#[cw_serde]
pub struct Config {
    pub code_ids: CodeIds,
    pub remote_opts: RemoteOpts,
    pub salt: String,
    pub subdenom: String,
    pub sdk_version: String,
    pub token_metadata: DenomMetadata,
}

#[cw_serde]
pub struct State {
    pub token_contract: String,
    pub core_contract: String,
    pub puppeteer_contract: String,
    pub withdrawal_voucher_contract: String,
    pub withdrawal_manager_contract: String,
    pub strategy_contract: String,
    pub validators_set_contract: String,
    pub distribution_contract: String,
    pub rewards_manager_contract: String,
}

#[cw_serde]
pub struct PauseInfoResponse {
    pub withdrawal_manager: drop_helpers::pause::PauseInfoResponse,
    pub core: drop_helpers::pause::PauseInfoResponse,
    pub rewards_manager: drop_helpers::pause::PauseInfoResponse,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
