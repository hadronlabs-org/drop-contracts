use cosmwasm_schema::write_api;

use lido_staking_base::{
    msg::token::{ExecuteMsg, InstantiateMsg, MigrateMsg},
    state::token::QueryMsg,
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
