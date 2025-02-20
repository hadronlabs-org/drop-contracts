use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimRewards { to_address: Option<String> },
}

#[cw_serde]
pub struct MigrateMsg {}
