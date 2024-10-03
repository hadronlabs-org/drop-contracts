use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};
use drop_puppeteer_base::state::RedeemShareItem;
use optfield::optfield;

#[optfield(pub ConfigOptional, attrs)]
#[cw_serde]
pub struct Config {
    pub puppeteer_contract: Addr,
    pub core_contract: Addr,
    pub validators_set_contract: Addr,
    pub port_id: String,
    pub transfer_channel_id: String,
    pub timeout: u64, // timeout for interchain transactions in seconds
    pub lsm_min_bond_amount: Uint128,
    pub lsm_redeem_threshold: u64,        //amount of lsm denoms
    pub lsm_redeem_maximum_interval: u64, //seconds
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
    Redeem { items: Vec<RedeemShareItem> },
    IBCTransfer { token: Coin, real_amount: Uint128 },
}

#[cw_serde]
#[derive(Default)]
pub struct TxState {
    pub status: TxStateStatus,
    pub transaction: Option<Transaction>,
}

pub const TX_STATE: Item<TxState> = Item::new("tx_state");
pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL_LSM_SHARES: Item<u128> = Item::new("total_lsm_shares_v0");
/// (local_denom, (remote_denom, shares_amount, real_amount))
pub const PENDING_LSM_SHARES: Map<String, (String, Uint128, Uint128)> =
    Map::new("pending_lsm_shares_v0");
/// (local_denom, (remote_denom, shares_amount, real_amount))
pub const LSM_SHARES_TO_REDEEM: Map<String, (String, Uint128, Uint128)> =
    Map::new("lsm_shares_to_redeem_v0");
pub const LAST_LSM_REDEEM: Item<u64> = Item::new("last_lsm_redeem_v0");

pub use reply_msg::ReplyMsg;
pub mod reply_msg {
    const OFFSET: u64 = u16::BITS as u64;
    pub const IBC_TRANSFER: u64 = 1 << OFFSET;
    pub const REDEEM: u64 = 2 << OFFSET;

    #[cosmwasm_schema::cw_serde]
    pub enum ReplyMsg {
        IbcTransfer,
        Redeem,
    }

    impl ReplyMsg {
        pub fn to_reply_id(&self) -> u64 {
            match self {
                ReplyMsg::IbcTransfer => IBC_TRANSFER,
                ReplyMsg::Redeem => REDEEM,
            }
        }

        pub fn from_reply_id(reply_id: u64) -> Self {
            match reply_id {
                IBC_TRANSFER => Self::IbcTransfer,
                REDEEM => Self::Redeem,
                _ => unreachable!(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn enum_variant_from_reply_id() {
            assert_eq!(ReplyMsg::from_reply_id(IBC_TRANSFER), ReplyMsg::IbcTransfer);
            assert_eq!(ReplyMsg::from_reply_id(REDEEM), ReplyMsg::Redeem);
        }

        #[test]
        fn enum_variant_to_reply_id() {
            assert_eq!(ReplyMsg::IbcTransfer.to_reply_id(), IBC_TRANSFER);
            assert_eq!(ReplyMsg::Redeem.to_reply_id(), REDEEM);
        }

        #[test]
        #[should_panic]
        fn invalid_reply_id() {
            ReplyMsg::from_reply_id(IBC_TRANSFER + 1);
        }
    }
}
