use cosmwasm_schema::write_api;

use lido_stargate_poc::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg},
    state::QueryMsg,
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
