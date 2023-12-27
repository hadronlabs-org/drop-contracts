use cosmwasm_schema::write_api;

use lido_staking_base::msg::hook_tester::{ExecuteMsg, InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
