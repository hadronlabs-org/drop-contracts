use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::Item;
use drop_helpers::ica::Ica;

#[cw_serde]
pub struct Config {
    pub port_id: String,
    pub transfer_channel_id: String,
    pub connection_id: String,
    pub timeout: u64,
    pub remote_denom: String,
    pub base_denom: String,
    pub allowed_senders: Vec<String>,
    pub puppeteer_ica: Option<String>,
    pub min_ibc_transfer: Uint128,
    pub min_staking_amount: Uint128,
}

#[cw_serde]
pub struct ConfigOptional {
    pub timeout: Option<u64>,
    pub allowed_senders: Option<Vec<String>>,
    pub puppeteer_ica: Option<String>,
    pub min_ibc_transfer: Option<Uint128>,
    pub min_staking_amount: Option<Uint128>,
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
    pub seq_id: Option<u64>,
    pub transaction: Option<Transaction>,
    pub reply_to: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("core");
pub const ICA: Ica = Ica::new("ica");
pub const ICA_ID: &str = "drop_STAKER";
pub const NON_STAKED_BALANCE: Item<Uint128> = Item::new("current_balance");
pub const TX_STATE: Item<TxState> = Item::new("tx_state");

pub use reply_msg::ReplyMsg;
mod reply_msg {
    const OFFSET: u64 = u16::BITS as u64;
    const SUDO_PAYLOAD: u64 = 1 << OFFSET;
    const IBC_TRANSFER: u64 = 2 << OFFSET;

    #[cosmwasm_schema::cw_serde]
    pub enum ReplyMsg {
        SudoPayload,
        IbcTransfer,
    }

    impl ReplyMsg {
        pub fn to_reply_id(&self) -> u64 {
            match self {
                ReplyMsg::SudoPayload => SUDO_PAYLOAD,
                ReplyMsg::IbcTransfer => IBC_TRANSFER,
            }
        }

        pub fn from_reply_id(reply_id: u64) -> Self {
            match reply_id {
                SUDO_PAYLOAD => Self::SudoPayload,
                IBC_TRANSFER => Self::IbcTransfer,
                _ => unreachable!(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn enum_variant_from_reply_id() {
            assert_eq!(ReplyMsg::from_reply_id(SUDO_PAYLOAD), ReplyMsg::SudoPayload);
            assert_eq!(ReplyMsg::from_reply_id(IBC_TRANSFER), ReplyMsg::IbcTransfer);
        }

        #[test]
        fn enum_variant_to_reply_id() {
            assert_eq!(ReplyMsg::SudoPayload.to_reply_id(), SUDO_PAYLOAD);
            assert_eq!(ReplyMsg::IbcTransfer.to_reply_id(), IBC_TRANSFER);
        }

        #[test]
        #[should_panic]
        fn invalid_reply_id() {
            ReplyMsg::from_reply_id(IBC_TRANSFER + 1);
        }
    }
}
