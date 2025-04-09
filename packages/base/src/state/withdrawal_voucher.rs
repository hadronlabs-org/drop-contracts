use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, Env, MessageInfo, Uint128};
use cw721::error::Cw721ContractError;
use cw721::traits::Cw721CustomMsg;
use cw721::traits::Cw721State;
use cw721::traits::{Contains, StateFactory};

#[cw_serde]
#[derive(Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
#[derive(Default)]
pub struct NftExtension {
    pub name: String,
    pub description: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub batch_id: String,
    pub amount: Uint128,
}

impl Cw721State for NftExtension {}

impl Contains for NftExtension {
    fn contains(&self, other: &Self) -> bool {
        if !other.name.is_empty() && self.name != other.name {
            return false;
        }

        if let Some(ref o_desc) = other.description {
            if self.description.as_ref() != Some(o_desc) {
                return false;
            }
        }

        if let Some(ref other_attrs) = other.attributes {
            if let Some(ref self_attrs) = self.attributes {
                for attr in other_attrs {
                    if !self_attrs.contains(attr) {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }

        if !other.batch_id.is_empty() && self.batch_id != other.batch_id {
            return false;
        }

        if other.amount != Uint128::zero() && self.amount != other.amount {
            return false;
        }
        true
    }
}

#[cw_serde]
#[derive(Default)]
pub struct NftExtensionMsg {
    pub name: String,
    pub description: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub batch_id: String,
    pub amount: Uint128,
}

impl Cw721CustomMsg for NftExtensionMsg {}

impl StateFactory<NftExtension> for NftExtensionMsg {
    fn create(
        &self,
        _deps: Deps,
        _env: &Env,
        _info: Option<&MessageInfo>,
        _current: Option<&NftExtension>,
    ) -> Result<NftExtension, Cw721ContractError> {
        Ok(NftExtension {
            name: self.name.clone(),
            description: self.description.clone(),
            attributes: self.attributes.clone(),
            batch_id: self.batch_id.clone(),
            amount: self.amount,
        })
    }

    fn validate(
        &self,
        _deps: Deps,
        _env: &Env,
        _info: Option<&MessageInfo>,
        _current: Option<&NftExtension>,
    ) -> Result<(), Cw721ContractError> {
        Ok(())
    }
}
