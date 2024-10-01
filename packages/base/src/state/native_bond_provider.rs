use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use drop_puppeteer_base::msg::ResponseHookMsg as PuppeteerResponseHookMsg;
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub base_denom: String,
    pub puppeteer_contract: Addr,
    pub core_contract: Addr,
    pub strategy_contract: Addr,
    pub min_ibc_transfer: Uint128,
    pub min_stake_amount: Uint128,
    pub port_id: String,
    pub transfer_channel_id: String,
    pub timeout: u64, // timeout for interchain transactions in seconds
}

#[cw_serde]
#[derive(Default)]
pub enum TxStateStatus {
    #[default]
    Idle,
    InProgress,
    WaitingForAck,
}

#[cw_serde]
pub enum Transaction {
    Stake { amount: Uint128 },
    IBCTransfer { amount: Uint128 },
}
#[cw_serde]
#[derive(Default)]
pub struct TxState {
    pub status: TxStateStatus,
    pub transaction: Option<Transaction>,
}

pub const TX_STATE: Item<TxState> = Item::new("tx_state");
pub const CONFIG: Item<Config> = Item::new("config");
pub const NON_STAKED_BALANCE: Item<Uint128> = Item::new("non_staked_balance");
pub const LAST_PUPPETEER_RESPONSE: Item<PuppeteerResponseHookMsg> =
    Item::new("last_puppeteer_response");

pub use reply_msg::ReplyMsg;
pub mod reply_msg {
    const OFFSET: u64 = u16::BITS as u64;
    pub const BANK_SEND: u64 = 1 << OFFSET;
    pub const IBC_TRANSFER: u64 = 2 << OFFSET;
    pub const BOND: u64 = 3 << OFFSET;

    #[cosmwasm_schema::cw_serde]
    pub enum ReplyMsg {
        BankSend,
        IbcTransfer,
        Bond,
    }

    impl ReplyMsg {
        pub fn to_reply_id(&self) -> u64 {
            match self {
                ReplyMsg::BankSend => BANK_SEND,
                ReplyMsg::IbcTransfer => IBC_TRANSFER,
                ReplyMsg::Bond => BOND,
            }
        }

        pub fn from_reply_id(reply_id: u64) -> Self {
            match reply_id {
                BANK_SEND => Self::BankSend,
                IBC_TRANSFER => Self::IbcTransfer,
                BOND => Self::Bond,
                _ => unreachable!(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn enum_variant_from_reply_id() {
            assert_eq!(ReplyMsg::from_reply_id(BANK_SEND), ReplyMsg::BankSend);
            assert_eq!(ReplyMsg::from_reply_id(IBC_TRANSFER), ReplyMsg::IbcTransfer);
            assert_eq!(ReplyMsg::from_reply_id(BOND), ReplyMsg::Bond);
        }

        #[test]
        fn enum_variant_to_reply_id() {
            assert_eq!(ReplyMsg::BankSend.to_reply_id(), BANK_SEND);
            assert_eq!(ReplyMsg::IbcTransfer.to_reply_id(), IBC_TRANSFER);
            assert_eq!(ReplyMsg::Bond.to_reply_id(), BOND);
        }

        #[test]
        #[should_panic]
        fn invalid_reply_id() {
            ReplyMsg::from_reply_id(IBC_TRANSFER + 1);
        }
    }
}
