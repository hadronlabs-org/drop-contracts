use cosmwasm_schema::write_api;

use lido_puppeteer_base::msg::QueryMsg;
use lido_staking_base::msg::puppeteer::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryExtMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg<QueryExtMsg>,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
