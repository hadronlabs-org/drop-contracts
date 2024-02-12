#![cfg(not(target_arch = "wasm32"))]

use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{OwnedDeps, Querier};
use neutron_sdk::bindings::query::NeutronQuery;
use std::marker::PhantomData;

pub fn mock_dependencies<Q: Querier + Default>() -> OwnedDeps<MockStorage, MockApi, Q, NeutronQuery>
{
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    }
}
