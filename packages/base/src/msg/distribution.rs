use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct Delegations {
    pub total: Uint128,
    pub total_weight: u64,
    pub delegations: Vec<Delegation>,
}
#[cw_serde]
pub struct Delegation {
    pub valoper_address: String,
    pub stake: Uint128,
    pub weight: u64,
}

#[cw_serde]
pub struct IdealDelegation {
    pub valoper_address: String,
    pub ideal_stake: Uint128,
    pub current_stake: Uint128,
    pub stake_change: Uint128,
    pub weight: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<(String, Uint128)>)]
    CalcDeposit {
        deposit: Uint128,
        delegations: Delegations,
    },
    #[returns(Vec<(String, Uint128)>)]
    CalcWithdraw {
        withdraw: Uint128,
        delegations: Delegations,
    },
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}
