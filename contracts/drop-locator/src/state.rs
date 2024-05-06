use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;
use cw_storage_plus::Map;

#[cw_serde]
pub struct Config {}

#[cw_serde]
pub struct ChainDetails {
    pub astroport_exchange_handler: String,
    pub auto_withdrawer: String,
    pub core: String,
    pub distribution: String,
    pub factory: String,
    pub hook_tester: String,
    pub proposal_votes_poc: String,
    pub provider_proposals_poc: String,
    pub pump: String,
    pub puppeteer: String,
    pub puppeteer_authz: String,
    pub rewards_manager: String,
    pub strategy: String,
    pub token: String,
    pub validators_set: String,
    pub validator_stats: String,
    pub withdrawal_manager: String,
    pub withdrawal_voucher: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Map<String, ChainDetails> = Map::new("state");
