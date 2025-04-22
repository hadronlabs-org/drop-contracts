use cosmwasm_schema::write_api;
use drop_staking_base::msg::neutron_distribution_mock::{ExecuteMsg, InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
