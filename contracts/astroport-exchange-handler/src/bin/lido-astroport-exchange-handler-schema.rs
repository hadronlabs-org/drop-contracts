use cosmwasm_schema::write_api;
use lido_staking_base::msg::{
    astroport_exchange_handler::QueryMsg,
    astroport_exchange_handler::{ExecuteMsg, InstantiateMsg, MigrateMsg},
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
