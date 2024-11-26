use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::Decimal256;
use cw_storage_plus::Item;

use drop_puppeteer_base::state::BalancesAndDelegationsState;
use drop_puppeteer_base::state::Transfer;
use neutron_sdk::interchain_queries::v045::types::Balances;

use crate::msg::puppeteer::MultiBalances;

#[cw_serde]
pub struct ConfigOptional {
    pub remote_denom: Option<String>,
    pub allowed_senders: Option<Vec<String>>,
    pub native_bond_provider: Option<Addr>,
}

#[cw_serde]
pub struct Config {
    pub remote_denom: String,
    pub allowed_senders: Vec<Addr>,
    pub delegations_queries_chunk_size: u32,
    pub native_bond_provider: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const NON_NATIVE_REWARD_BALANCES: Item<BalancesAndDelegationsState<MultiBalances>> =
    Item::new("non_native_reward_balances");

pub const RECIPIENT_TRANSFERS: Item<Vec<Transfer>> = Item::new("recipient_transfers");

pub const DECIMAL_PLACES: u32 = 18;

#[cw_serde]
pub struct BalancesAndDelegations {
    pub balances: Balances,
    pub delegations: Delegations,
}

#[cw_serde]
pub struct Delegations {
    pub delegations: Vec<DropDelegation>,
}

#[cw_serde]
pub struct DropDelegation {
    pub delegator: Addr,
    /// A validator address (e.g. cosmosvaloper1...)
    pub validator: String,
    /// How much we have locked in the delegation
    pub amount: cosmwasm_std::Coin,
    /// How many shares the delegator has in the validator
    pub share_ratio: Decimal256,
}

pub mod reply_msg {
    const OFFSET: u64 = u16::BITS as u64;
    pub const DELEGATE: u64 = 1 << OFFSET;
    pub const UNDELEGATE: u64 = 2 << OFFSET;

    #[cosmwasm_schema::cw_serde]
    pub enum ReplyMsg {
        Delegate,
        Undelegate,
    }

    impl ReplyMsg {
        pub fn to_reply_id(&self) -> u64 {
            match self {
                ReplyMsg::Delegate => DELEGATE,
                ReplyMsg::Undelegate => UNDELEGATE,
            }
        }

        pub fn from_reply_id(reply_id: u64) -> Self {
            match reply_id {
                DELEGATE => Self::Delegate,
                UNDELEGATE => Self::Undelegate,
                _ => unreachable!(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn enum_variant_from_reply_id() {
            assert_eq!(ReplyMsg::from_reply_id(DELEGATE), ReplyMsg::Delegate);
            assert_eq!(ReplyMsg::from_reply_id(UNDELEGATE), ReplyMsg::Undelegate);
        }

        #[test]
        fn enum_variant_to_reply_id() {
            assert_eq!(ReplyMsg::Delegate.to_reply_id(), DELEGATE);
            assert_eq!(ReplyMsg::Undelegate.to_reply_id(), UNDELEGATE);
        }
    }
}
