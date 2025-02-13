use crate::state::ConfigOptional;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub core_contract: String,
    pub withdrawal_manager: String,
    pub withdrawal_voucher: String,
    pub source_port: String,
    pub source_channel: String,
    pub ibc_timeout: u64,
    pub ibc_denom: String,
    pub prefix: String,
    pub retry_limit: u64,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig { new_config: ConfigOptional },
    Withdraw { receiver: String },
    Unbond { receiver: String },
    Retry { receiver: String },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}

/// FungibleTokenPacketData defines a struct for the packet payload
/// See FungibleTokenPacketData spec:
/// <https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#data-structures>
#[cw_serde]
pub struct FungibleTokenPacketData {
    /// the token denomination to be transferred
    pub denom: String,
    /// the token amount to be transferred
    pub amount: String,
    /// the sender address
    pub sender: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// optional memo
    pub memo: Option<String>,
}
