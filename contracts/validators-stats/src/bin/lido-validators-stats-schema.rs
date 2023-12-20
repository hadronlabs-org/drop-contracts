use cosmwasm_schema::write_api;

use lido_staking_base::{
    msg::validatorsstats::{ExecuteMsg, InstantiateMsg, MigrateMsg},
    state::validatorsstats::QueryMsg,
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
