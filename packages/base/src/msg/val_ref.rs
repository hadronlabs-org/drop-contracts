#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub core_address: String,
    pub validators_set_address: String,
    pub owner: String,
}

#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Ref)]
    Ref { r#ref: String },
    #[returns(Vec<Ref>)]
    AllRefs {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub core_address: String,
    pub validators_set_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct Ref {
    pub r#ref: String,
    pub validator_address: String,
}

#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    BondCallback(crate::msg::core::BondHook),
    UpdateConfig {
        core_address: String,
        validators_set_address: String,
    },
    SetRefs {
        refs: Vec<Ref>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
