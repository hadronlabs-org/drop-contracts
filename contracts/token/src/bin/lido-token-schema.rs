use cosmwasm_schema::write_api;
use lido_staking_base::msg::TokenExecuteMsg;
use lido_token::{InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: TokenExecuteMsg,
        migrate: MigrateMsg
    }
}
