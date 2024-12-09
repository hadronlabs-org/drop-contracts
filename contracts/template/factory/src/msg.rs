use cosmwasm_schema::{cw_serde, QueryResponses};
use drop_staking_base::state::factory::State as FactoryState;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateState { state: FactoryState },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryState)]
    State {},
}
