use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::Decimal256;
use cosmwasm_std::Uint128;
use cw_storage_plus::Item;

use drop_puppeteer_base::state::BalancesAndDelegationsState;

use crate::msg::puppeteer::MultiBalances;

use super::puppeteer::DropDelegation;

#[cw_serde]
pub struct ConfigOptional {
    pub allowed_senders: Option<Vec<String>>,
    pub distribution_module_contract: Option<String>,
}

#[cw_serde]
pub struct Config {
    pub allowed_senders: Vec<Addr>,
    pub distribution_module_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const NON_NATIVE_REWARD_BALANCES: Item<BalancesAndDelegationsState<MultiBalances>> =
    Item::new("non_native_reward_balances");

pub const DECIMAL_PLACES: u32 = 18;

pub const REWARDS_WITHDRAW_ADDR: Item<Addr> = Item::new("rewards_withdraw_addr");
#[cw_serde]
pub struct PageResponse {
    pub next_key: Option<Vec<u8>>,
    pub total: Uint128,
}

#[cw_serde]
pub struct Delegation {
    pub delegator_address: Addr,
    pub validator_address: String,
    pub shares: Decimal256,
}

#[cw_serde]
pub struct DelegationResponseNative {
    pub delegation: Delegation,
    pub balance: cosmwasm_std::Coin,
}

impl From<DelegationResponseNative> for DropDelegation {
    fn from(delegation: DelegationResponseNative) -> Self {
        DropDelegation {
            delegator: delegation.delegation.delegator_address,
            validator: delegation.delegation.validator_address,
            amount: delegation.balance,
            share_ratio: delegation.delegation.shares,
        }
    }
}

#[cw_serde]
pub struct QueryDelegationResponse {
    pub delegation_response: Option<DelegationResponseNative>,
}

#[cw_serde]
pub struct QueryDelegatorDelegationsResponse {
    pub delegation_responses: Vec<DelegationResponseNative>,
    pub pagination: PageResponse,
}

pub mod unbonding_delegations {
    use cosmwasm_std::{StdError, StdResult, Timestamp, Uint128, Uint64};
    use drop_puppeteer_base::state::UnbondingDelegation;
    use neutron_sdk::interchain_queries::v047::types::UnbondingEntry;
    use time::{format_description::well_known::Rfc3339, PrimitiveDateTime};

    use super::PageResponse;

    #[cosmwasm_schema::cw_serde]
    pub struct UnbondingDelegationEntry {
        pub creation_height: Uint64,
        pub completion_time: Option<String>,
        pub initial_balance: Uint128,
        pub balance: Uint128,
        pub unbonding_id: Uint128,
        pub unbonding_on_hold_ref_count: Uint128,
    }

    impl TryFrom<UnbondingDelegationEntry> for UnbondingEntry {
        type Error = StdError;

        fn try_from(res: UnbondingDelegationEntry) -> StdResult<Self> {
            if res.completion_time.is_none() {
                return Err(StdError::generic_err("completion_time is not set"));
            }
            let completion_time = convert_datetime_str_to_timestamp(&res.completion_time.unwrap())?;

            Ok(UnbondingEntry {
                creation_height: res.creation_height.u64(),
                completion_time: Some(completion_time),
                initial_balance: res.initial_balance,
                balance: res.balance,
            })
        }
    }

    #[cosmwasm_schema::cw_serde]
    pub struct UnbondingDelegationNative {
        pub delegator_address: String,
        pub validator_address: String,
        pub entries: Vec<UnbondingDelegationEntry>,
    }

    #[cosmwasm_schema::cw_serde]
    pub struct QueryDelegatorUnbondingDelegationsResponse {
        pub unbonding_responses: Vec<UnbondingDelegationNative>,
        pub pagination: PageResponse,
    }

    impl TryFrom<QueryDelegatorUnbondingDelegationsResponse> for Vec<UnbondingDelegation> {
        type Error = StdError;

        fn try_from(res: QueryDelegatorUnbondingDelegationsResponse) -> StdResult<Self> {
            let mut unbonding_delegations: Vec<UnbondingDelegation> = vec![];

            for unbonding_response in res.unbonding_responses {
                let entries = unbonding_response
                    .entries
                    .into_iter()
                    .map(UnbondingEntry::try_from)
                    .collect::<StdResult<Vec<UnbondingEntry>>>()?;

                let unbonding_delegation = UnbondingDelegation {
                    validator_address: unbonding_response.validator_address,
                    query_id: 0,
                    last_updated_height: 0,
                    unbonding_delegations: entries,
                };

                unbonding_delegations.push(unbonding_delegation);
            }

            Ok(unbonding_delegations)
        }
    }

    fn convert_datetime_str_to_timestamp(datetime_str: &str) -> StdResult<Timestamp> {
        let primitive_datetime = PrimitiveDateTime::parse(datetime_str, &Rfc3339)
            .map_err(|_| StdError::generic_err("Unable to parse format description"))?;

        let datetime = primitive_datetime.assume_utc();

        Ok(Timestamp::from_seconds(datetime.unix_timestamp() as u64))
    }
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
