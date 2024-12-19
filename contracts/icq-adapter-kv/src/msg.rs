use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Options {
    pub delegations_queries_chunk_size: u32,
    pub connection_id: String,
    pub sdk_version: String,
    pub update_period: u64,
}
