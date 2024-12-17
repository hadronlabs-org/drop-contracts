use crate::state::icq_adapter::ConfigOptional;
use cosmwasm_schema::cw_serde;

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(cosmwasm_schema::QueryResponses)]
pub enum QueryMsg<E> {
    #[returns(crate::state::icq_router::Config)]
    Config {},
    #[returns(E)]
    Extention(E),
}

#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg<E> {
    UpdateValidatorSet { validators: Vec<String> },
    UpdateConfig { new_config: ConfigOptional<E> },
}

#[cw_serde]
pub struct InstantiateMsg<O> {
    pub router: String,
    pub ica: String,
    pub remote_denom: String,
    pub owner: Option<String>,
    pub options: O,
}

#[cw_serde]
pub struct MigrateMsg {}
