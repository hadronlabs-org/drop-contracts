use neutron_sdk::{bindings::types::StorageValue, NeutronResult};

pub trait PuppeteerReconstruct {
    fn reconstruct(storage_values: &[StorageValue], version: &str) -> NeutronResult<Self>
    where
        Self: Sized;
}
