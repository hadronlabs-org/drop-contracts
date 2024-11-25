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

#[cw_serde]
pub enum ExecuteMsg {
    UpdateValidatorSet { validators: Vec<String> },
    UpdateBalances {},
    UpdateDelegations {},
    UpdateConfig {},
}

#[cw_serde]
pub struct InstantiateMsg<O> {
    pub router: String,
    pub owner: Option<String>,
    pub opts: O,
}

#[cw_serde]
pub struct MigrateMsg {}
