use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Empty, Uint128};
use schemars::JsonSchema;

#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    RegisterQuery {},
}

#[cw_serde]
pub struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg<E = Empty>
where
    E: JsonSchema,
{
    #[returns(crate::state::ConfigResponse)]
    Config {},
    #[returns(drop_helpers::ica::IcaState)]
    Ica {},
    #[returns(Vec<crate::peripheral_hook::Transaction>)]
    Transactions {},
    #[returns(Vec<(u64, String)>)]
    KVQueryIds {},
    #[returns(cosmwasm_std::Binary)]
    Extension { msg: E },
    #[returns(crate::state::TxState)]
    TxState {},
}

#[cw_serde]
pub struct TransferReadyBatchesMsg {
    pub batch_ids: Vec<u128>,
    pub emergency: bool,
    pub amount: Uint128,
    pub recipient: String,
}
