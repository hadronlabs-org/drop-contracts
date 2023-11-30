use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Validator {
    pub valoper_address: String,
    pub valcons_address: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub connection_id: String,
    pub port_id: String,
    pub profile_update_period: u64,
    pub info_update_period: u64,
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterStatsQueries { validators: Vec<Validator> },
}

#[cw_serde]
pub struct MigrateMsg {}
