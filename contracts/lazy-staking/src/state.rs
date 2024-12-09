use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, DenomMetadata, Deps, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct SplittingTarget {
    pub addr: Addr,
    pub unbonding_weight: Uint128,
}

#[cw_serde]
pub struct Config {
    pub factory_addr: String,
    pub base_denom: String,
    pub splitting_targets: Vec<SplittingTarget>,
}

impl Config {
    pub fn new(
        factory_addr: String,
        base_denom: String,
        splitting_targets: Vec<SplittingTarget>,
    ) -> Self {
        Self {
            factory_addr,
            base_denom,
            splitting_targets,
        }
    }

    pub fn validate_base_denom(&self, deps: Deps) -> Result<(), ContractError> {
        let total_supply = deps.querier.query_supply(self.base_denom.clone());
        if total_supply.is_err() || total_supply.unwrap().amount.is_zero() {
            return Ok(());
        }
        Err(ContractError::BaseDenomError {})
    }

    pub fn validate_splitting_targets(&self, deps: Deps) -> Result<(), ContractError> {
        let mut accum: Uint128 = Uint128::zero();
        for target in &self.splitting_targets {
            accum = accum
                .checked_add(target.unbonding_weight)
                .map_err(ContractError::OverflowError)?;
            deps.api
                .addr_validate(target.addr.as_str())
                .map_err(|_| ContractError::InvalidAddressProvided {})?;
        }
        Ok(())
    }

    pub fn validate_factory_addr(&self, deps: Deps) -> Result<(), ContractError> {
        if deps.api.addr_validate(self.factory_addr.as_str()).is_err() {
            return Err(ContractError::InvalidAddressProvided {});
        }
        Ok(())
    }
}

pub const CREATE_DENOM_REPLY_ID: u64 = 1;

pub const CONFIG: Item<Config> = Item::new("config");
pub const EXCHANGE_RATE: Item<Decimal> = Item::new("exchange_rate");
pub const REWARDS_RATE: Item<Decimal> = Item::new("rewards_rate");

pub const TOKEN_METADATA: Item<DenomMetadata> = Item::new("token_metadata");
pub const DENOM: Item<String> = Item::new("denom");
pub const EXPONENT: Item<u32> = Item::new("exponent");
