use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub core_contract: String,
    pub source_port: String,
    pub source_channel: String,
    pub ibc_timeout: u64,
    pub prefix: String,
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Bond {
        receiver: String,
        r#ref: Option<String>,
        backup: Option<String>,
    },
    UpdateConfig {
        new_config: crate::state::mirror::ConfigOptional,
    },
    Complete {
        items: Vec<u64>,
    },
    ChangeReturnType {
        id: u64,
        return_type: crate::state::mirror::ReturnType,
    },
    // by admin
    UpdateBond {
        id: u64,
        receiver: String,
        backup: Option<String>,
        return_type: crate::state::mirror::ReturnType,
    },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::mirror::Config)]
    Config {},

    #[returns(crate::state::mirror::BondItem)]
    One { id: u64 },

    #[returns(Vec<crate::state::mirror::BondItem>)]
    All {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
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
    pub memo: String,
}
