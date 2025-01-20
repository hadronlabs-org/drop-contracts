use crate::error::core::{ContractError, ContractResult};
use cosmwasm_std::{Addr, Empty, StdResult, Storage};
use cw_storage_plus::{Item, Map};

pub struct BondProviders {
    pub providers: Map<Addr, Empty>,
    pub next_provider_ptr: Item<u64>,
}

impl BondProviders {
    pub const fn new(storage_key: &'static str, pointer_storage_key: &'static str) -> Self {
        Self {
            providers: Map::new(storage_key),
            next_provider_ptr: Item::new(pointer_storage_key),
        }
    }

    pub fn init(&self, storage: &mut dyn Storage) -> StdResult<()> {
        self.next_provider_ptr.save(storage, &0)?;
        Ok(())
    }

    pub fn get_all_providers(&self, storage: &dyn Storage) -> StdResult<Vec<Addr>> {
        let providers = self
            .providers
            .range(storage, None, None, cosmwasm_std::Order::Ascending)
            .map(|x| x.map(|(addr, _)| addr))
            .collect::<StdResult<Vec<_>>>()?;

        Ok(providers)
    }

    pub fn add(&self, storage: &mut dyn Storage, provider: Addr) -> ContractResult<()> {
        if self.providers.has(storage, provider.clone()) {
            return Err(ContractError::BondProviderAlreadyExists {});
        }

        let empty = Empty {};
        self.providers.save(storage, provider.clone(), &empty)?;
        Ok(())
    }

    pub fn remove(&self, storage: &mut dyn Storage, provider: Addr) -> ContractResult<()> {
        self.providers.remove(storage, provider);
        Ok(())
    }

    pub fn next(&self, storage: &mut dyn Storage) -> ContractResult<Addr> {
        let mut next_provider_ptr = self.next_provider_ptr.load(storage)?;
        let providers = self.get_all_providers(storage)?;

        let total_providers = providers.len() as u64;
        if total_providers == 0 {
            return Err(ContractError::BondProvidersListAreEmpty {});
        }

        if next_provider_ptr >= total_providers {
            next_provider_ptr = 0;
        }

        self.next_provider_ptr
            .save(storage, &(next_provider_ptr + 1))?;

        Ok(providers[next_provider_ptr as usize].clone())
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::MockStorage, Addr};

    use crate::error::core::ContractError;

    use super::BondProviders;

    #[test]
    fn get_empty_providers_list() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");
        let providers = bond_providers
            .get_all_providers(&MockStorage::default())
            .unwrap();
        assert_eq!(providers.len(), 0);
    }

    #[test]
    fn add_one_provider_to_list() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();

        bond_providers
            .add(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        let providers = bond_providers.get_all_providers(storage).unwrap();
        assert_eq!(providers.len(), 1);
    }

    #[test]
    fn add_two_providers_to_list() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();

        bond_providers
            .add(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        bond_providers
            .add(storage, Addr::unchecked("lsm_share_provider_address"))
            .unwrap();
        let providers = bond_providers.get_all_providers(storage).unwrap();
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn remove_one_provider_from_list() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();

        bond_providers
            .add(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        bond_providers
            .add(storage, Addr::unchecked("lsm_share_provider_address"))
            .unwrap();
        bond_providers
            .remove(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        let providers = bond_providers.get_all_providers(storage).unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0], Addr::unchecked("lsm_share_provider_address"));
    }

    #[test]
    fn remove_two_providers_from_list() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();

        bond_providers
            .add(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        bond_providers
            .add(storage, Addr::unchecked("lsm_share_provider_address"))
            .unwrap();
        bond_providers
            .remove(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        bond_providers
            .remove(storage, Addr::unchecked("lsm_share_provider_address"))
            .unwrap();
        let providers = bond_providers.get_all_providers(storage).unwrap();
        assert_eq!(providers.len(), 0);
    }

    #[test]
    fn error_on_same_provider() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();

        bond_providers
            .add(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        let err = bond_providers
            .add(storage, Addr::unchecked("native_provider_address"))
            .unwrap_err();

        assert_eq!(err, ContractError::BondProviderAlreadyExists {});
    }

    #[test]
    fn error_empty_providers_list_iterate() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();
        bond_providers.init(storage).unwrap();

        let err = bond_providers.next(storage).unwrap_err();

        assert_eq!(err, ContractError::BondProvidersListAreEmpty {});
    }

    #[test]
    fn add_one_provider_to_list_and_iterate() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();
        bond_providers.init(storage).unwrap();

        bond_providers
            .add(storage, Addr::unchecked("native_provider_address"))
            .unwrap();
        let provider = bond_providers.next(storage).unwrap();
        assert_eq!(provider, Addr::unchecked("native_provider_address"));

        let provider = bond_providers.next(storage).unwrap();
        assert_eq!(provider, Addr::unchecked("native_provider_address"));

        let provider = bond_providers.next(storage).unwrap();
        assert_eq!(provider, Addr::unchecked("native_provider_address"));
    }

    #[test]
    fn add_two_providers_to_list_and_iterate() {
        let bond_providers: BondProviders =
            BondProviders::new("bond_providers", "bond_providers_ptr");

        let storage = &mut MockStorage::default();
        bond_providers.init(storage).unwrap();

        bond_providers
            .add(storage, Addr::unchecked("1_provider_address"))
            .unwrap();
        bond_providers
            .add(storage, Addr::unchecked("2_provider_address"))
            .unwrap();

        let provider = bond_providers.next(storage).unwrap();
        assert_eq!(provider, Addr::unchecked("1_provider_address"));

        let provider = bond_providers.next(storage).unwrap();
        assert_eq!(provider, Addr::unchecked("2_provider_address"));

        let provider = bond_providers.next(storage).unwrap();
        assert_eq!(provider, Addr::unchecked("1_provider_address"));

        let provider = bond_providers.next(storage).unwrap();
        assert_eq!(provider, Addr::unchecked("2_provider_address"));
    }
}
