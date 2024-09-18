use cosmwasm_schema::write_api;

use drop_puppeteer_base::msg::QueryMsg;
use drop_staking_base::msg::puppeteer_initia::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryExtMsg,
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg<QueryExtMsg>,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
