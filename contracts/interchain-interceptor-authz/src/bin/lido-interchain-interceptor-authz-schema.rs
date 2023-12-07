use cosmwasm_schema::write_api;

use lido_interchain_interceptor_authz::{
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
