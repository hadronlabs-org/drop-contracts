use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub core_contract: String,
    pub withdrawal_manager: String,
    pub withdrawal_voucher: String,
    pub source_port: String,
    pub source_channel: String,
    pub ibc_timeout: u64,
    pub prefix: String,
    pub retry_limit: u64,
}

#[cw_serde]
pub struct ConfigOptional {
    pub core_contract: Option<String>,
    pub withdrawal_manager: Option<String>,
    pub withdrawal_voucher: Option<String>,
    pub source_port: Option<String>,
    pub source_channel: Option<String>,
    pub ibc_timeout: Option<u64>,
    pub prefix: Option<String>,
    pub retry_limit: Option<u64>,
}

#[cw_serde]
pub struct TimeoutRange {
    pub from: u64,
    pub to: u64,
}

const TIMEOUT_30D: u64 = 2592000;
pub const IBC_TRANSFER_SUDO_REPLY_ID: u64 = u64::MAX;

pub const CONFIG: Item<Config> = Item::new("config");
pub const SUDO_SEQ_ID_TO_COIN: Map<u64, Coin> = Map::new("sudo_seq_id_to_coin");
pub const REPLY_TRANSFER_COIN: Item<Coin> = Item::new("reply_transfer_coin");
pub const UNBOND_REPLY_ID: Item<u64> = Item::new("unbond_reply_id");
pub const WITHDRAW_REPLY_ID: Item<u64> = Item::new("withdraw_reply_id");
// Do we really need this map?
pub const REPLY_RECEIVERS: Map<u64, String> = Map::new("reply_receivers");
pub const FAILED_TRANSFERS: Map<String, Vec<Coin>> = Map::new("failed_transfers");
pub const TF_DENOM_TO_NFT_ID: Map<String, String> = Map::new("tf_denom_to_nft_id");
pub const TIMEOUT_RANGE: TimeoutRange = TimeoutRange {
    from: 0,
    to: TIMEOUT_30D,
};
