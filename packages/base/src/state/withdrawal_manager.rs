use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub core_contract: Addr,
    pub withdrawal_voucher_contract: Addr,
    pub base_denom: String,
}

#[cw_serde]
pub enum PauseType {
    Switch { receive_nft_withdraw: bool },
    Height { receive_nft_withdraw: u64 },
}

impl Default for PauseType {
    fn default() -> Self {
        PauseType::Switch {
            receive_nft_withdraw: false,
        }
    }
}

#[cw_serde]
#[derive(Default)]
pub struct Pause {
    pub pause: PauseType,
}

pub type Cw721ReceiveMsg = cw721::Cw721ReceiveMsg;
pub const PAUSE: Item<Pause> = Item::new("pause");
pub const CONFIG: Item<Config> = Item::new("config");
