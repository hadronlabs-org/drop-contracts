use cosmwasm_schema::write_api;
use lido_core::{
    msg::{ExecuteMsg, MigrateMsg},
    state::QueryMsg,
};
use lido_staking_base::msg::CoreInstantiateMsg;

fn main() {
    write_api! {
        instantiate: CoreInstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
