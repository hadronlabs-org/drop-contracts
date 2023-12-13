use cosmwasm_schema::write_api;
use lido_staking_base::msg::{TokenExecuteMsg, TokenInstantiateMsg};
use lido_token::{MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: TokenInstantiateMsg,
        query: QueryMsg,
        execute: TokenExecuteMsg,
        migrate: MigrateMsg
    }
}
