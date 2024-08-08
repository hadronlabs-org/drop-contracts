use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, CosmosMsg, StdError, StdResult, Storage};
use cw_storage_plus::Item;
use neutron_sdk::bindings::msg::NeutronMsg;

#[cw_serde]
#[derive(Default)]
pub enum IcaState {
    #[default]
    None,
    InProgress,
    Timeout,
    Registered {
        ica_address: String,
        port_id: String,
        channel_id: String,
    },
}

pub struct Ica<'a>(Item<'a, IcaState>);

impl<'a> Ica<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        Self(Item::new(storage_key))
    }

    pub fn load(&self, store: &dyn Storage) -> StdResult<IcaState> {
        self.0.may_load(store).map(Option::unwrap_or_default)
    }

    pub fn register(
        &self,
        store: &mut dyn Storage,
        connection_id: impl Into<String>,
        ica_id: impl Into<String>,
        register_fee: Coin,
    ) -> StdResult<CosmosMsg<NeutronMsg>> {
        match self.load(store)? {
            IcaState::InProgress => Err(StdError::generic_err(
                "ICA registration is in progress right now",
            )),
            IcaState::Registered { .. } => Err(StdError::generic_err("ICA is already registered")),
            IcaState::Timeout | IcaState::None => {
                self.0.save(store, &IcaState::InProgress)?;
                Ok(NeutronMsg::register_interchain_account(
                    connection_id.into(),
                    ica_id.into(),
                    Some(vec![register_fee]),
                )
                .into())
            }
        }
    }

    pub fn set_timeout(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.0.save(store, &IcaState::Timeout)
    }

    pub fn set_address(
        &self,
        store: &mut dyn Storage,
        address: impl Into<String>,
        port_id: impl Into<String>,
        channel_id: impl Into<String>,
    ) -> StdResult<()> {
        self.0.save(
            store,
            &IcaState::Registered {
                ica_address: address.into(),
                port_id: port_id.into(),
                channel_id: channel_id.into(),
            },
        )
    }

    pub fn get_address(&self, store: &dyn Storage) -> StdResult<String> {
        match self.load(store)? {
            IcaState::Registered {
                ica_address,
                port_id: _,
                channel_id: _,
            } => Ok(ica_address),
            IcaState::None => Err(StdError::generic_err(
                "Interchain account is not registered. Please register it first",
            )),
            IcaState::InProgress => Err(StdError::generic_err(
                "Interchain account registration in progress. Please wait until it is finished",
            )),
            IcaState::Timeout => Err(StdError::generic_err(
                "Interchain account registration timed out. Please register it again",
            )),
        }
    }
}
