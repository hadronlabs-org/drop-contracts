use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

use crate::msg::icq_router::{BalancesData, DelegationsData};

#[cw_serde]
pub struct Config<E> {
    pub router: Addr,
    pub ica: String,
    pub remote_denom: String,
    pub options: E,
}

#[cw_serde]
pub struct ConfigOptional<E> {
    pub router: Option<String>,
    pub ica: Option<String>,
    pub remote_denom: Option<String>,
    pub options: Option<E>,
}

pub struct IcqAdapter<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub config: Item<'a, Config<T>>,
    pub balances: Item<'a, BalancesData>,
    pub delegations: Item<'a, DelegationsData>,
}

impl<T> Default for IcqAdapter<'static, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IcqAdapter<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new() -> Self {
        Self {
            config: Item::new("config"),
            balances: Item::new("balances"),
            delegations: Item::new("delegations"),
        }
    }
}
