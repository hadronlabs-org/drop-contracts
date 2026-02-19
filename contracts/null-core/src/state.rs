use cosmwasm_std::Uint64;
use cw_storage_plus::{Item, Map};
use drop_puppeteer_base::peripheral_hook::ResponseHookMsg as PuppeteerResponseHookMsg;

pub const NEXT_MESSAGE_ID: Item<Uint64> = Item::new("next_message_id");
pub const SAVED_MESSAGES: Map<u64, (String, PuppeteerResponseHookMsg)> = Map::new("saved_messages");
