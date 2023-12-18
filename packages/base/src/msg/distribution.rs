use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

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
    #[returns(Vec<IdealDelegation>)]
    CalcDeposit {
        deposit: Uint128,
        delegations: Vec<Delegation>,
    },
    #[returns(Vec<IdealDelegation>)]
    CalcWithdraw {
        withdraw: Uint128,
        delegations: Vec<Delegation>,
    },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub strategy_address: String,
    pub core_address: String,
}

#[cw_serde]
pub enum MigrateMsg {}
