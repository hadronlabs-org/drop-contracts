use cw_storage_plus::Map;

pub const DELEGATIONS_AND_BALANCES_QUERY_ID_CHUNK: Map<u64, u64> =
    Map::new("delegations_and_balances_query_id_chunk");
