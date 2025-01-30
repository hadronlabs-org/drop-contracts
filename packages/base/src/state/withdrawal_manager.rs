use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use drop_helpers::pause::Interval;

#[cw_serde]
pub struct Config {
    pub factory_contract: Addr,
    pub base_denom: String,
}

#[cw_serde]
#[derive(Default)]
pub struct Pause {
    pub receive_nft_withdraw: Interval,
}

pub type Cw721ReceiveMsg = cw721::Cw721ReceiveMsg;
pub const PAUSE: Item<Pause> = Item::new("pause");
pub const CONFIG: Item<Config> = Item::new("config");
