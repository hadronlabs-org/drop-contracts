use cosmwasm_schema::write_api;

use lido_interchain_interceptor::{
    msg::{ExecuteMsg, MigrateMsg},
    state::QueryMsg,
};
use lido_interchain_interceptor_base::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
