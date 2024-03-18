use cosmwasm_schema::write_api;
use drop_staking_base::msg::{
    rewards_manager::QueryMsg,
    rewards_manager::{ExecuteMsg, InstantiateMsg, MigrateMsg},
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
