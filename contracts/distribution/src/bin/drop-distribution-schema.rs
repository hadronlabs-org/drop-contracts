use cosmwasm_schema::write_api;
use drop_staking_base::msg::distribution::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg
    }
}
