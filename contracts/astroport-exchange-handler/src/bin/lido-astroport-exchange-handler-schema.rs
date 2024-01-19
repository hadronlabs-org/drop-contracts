use cosmwasm_schema::write_api;
use lido_staking_base::msg::{
    distribution::QueryMsg,
    strategy::{ExecuteMsg, InstantiateMsg, MigrateMsg},
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
