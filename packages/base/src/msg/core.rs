use crate::{
    error::core::ContractResult,
    state::core::{Config, ConfigOptional, Pause},
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[allow(unused_imports)]
use cosmwasm_std::{Addr, Deps, Uint128, Uint64};
use cw_ownable::cw_ownable_execute;
use drop_puppeteer_base::peripheral_hook::ResponseHookMsg as PuppeteerResponseHookMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_contract: String,
    pub base_denom: String,
    pub remote_denom: String,
    pub idle_min_interval: u64,        //seconds
    pub unbonding_period: u64,         //seconds
    pub unbonding_safe_period: u64,    //seconds
    pub unbond_batch_switch_time: u64, //seconds
    pub pump_ica_address: Option<String>,
    pub owner: String,
    pub emergency_address: Option<String>,
    pub icq_update_delay: u64, // blocks
}

impl InstantiateMsg {
    pub fn into_config(self, deps: Deps) -> ContractResult<Config> {
        Ok(Config {
            factory_contract: deps.api.addr_validate(&self.factory_contract)?,
            base_denom: self.base_denom,
            remote_denom: self.remote_denom,
            idle_min_interval: self.idle_min_interval,
            unbonding_safe_period: self.unbonding_safe_period,
            unbonding_period: self.unbonding_period,
            pump_ica_address: self.pump_ica_address,
            unbond_batch_switch_time: self.unbond_batch_switch_time,
            emergency_address: self.emergency_address,
            icq_update_delay: self.icq_update_delay,
        })
    }
}

#[cw_serde]
pub struct LastPuppeteerResponse {
    pub response: Option<PuppeteerResponseHookMsg>,
}

#[cw_serde]
pub struct FailedBatchResponse {
    pub response: Option<u128>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(String)]
    Owner {},
    #[returns(cosmwasm_std::Decimal)]
    ExchangeRate {},
    #[returns(Uint128)]
    CurrentUnbondBatch {},
    #[returns(crate::state::core::UnbondBatch)]
    UnbondBatch { batch_id: Uint128 },
    #[returns(crate::state::core::UnbondBatchesResponse)]
    UnbondBatches {
        limit: Option<Uint64>,
        page_key: Option<Uint128>,
    },
    #[returns(crate::state::core::ContractState)]
    ContractState {},
    #[returns(LastPuppeteerResponse)]
    LastPuppeteerResponse {},
    #[returns(Uint128)]
    TotalBonded {},
    #[returns(Vec<Addr>)]
    BondProviders {},
    #[returns(Uint128)]
    TotalAsyncTokens {},
    #[returns(FailedBatchResponse)]
    FailedBatch {},
    #[returns(Pause)]
    Pause {},
    #[returns(Vec<String>)]
    BondHooks {},
}

#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    Bond {
        receiver: Option<String>,
        r#ref: Option<String>,
    },
    Unbond {},
    Tick {},
    //permissioned
    AddBondProvider {
        bond_provider_address: String,
    },
    RemoveBondProvider {
        bond_provider_address: String,
    },
    UpdateConfig {
        new_config: Box<ConfigOptional>,
    },
    UpdateWithdrawnAmount {
        batch_id: u128,
        withdrawn_amount: Uint128,
    },
    PeripheralHook(Box<PuppeteerResponseHookMsg>),
    ProcessEmergencyBatch {
        batch_id: u128,
        unbonded_amount: Uint128,
    },
    SetPause(Pause),
    SetBondHooks {
        hooks: Vec<String>,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct BondHook {
    pub sender: Addr,
    pub denom: String,
    pub amount: Uint128,
    pub dasset_minted: Uint128,
    pub r#ref: Option<String>,
}

// Contracts receiving bond hooks are expected to have
// `BondCallback(BondHook)` in their `ExecuteMsg`
#[cw_serde]
pub enum BondCallback {
    BondCallback(BondHook),
}

#[cw_serde]
pub struct VoucherTrait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
#[derive(Default)]
pub struct VoucherMetadata {
    pub name: String,
    pub description: Option<String>,
    pub attributes: Option<Vec<VoucherTrait>>,
    pub batch_id: String,
    pub amount: Uint128,
}

pub type VoucherExtension = Option<VoucherMetadata>;

#[cw_serde]
pub struct VoucherMintMsg {
    /// Unique ID of the NFT
    pub token_id: String,
    /// The owner of the newly minter NFT
    pub owner: String,
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,
    /// Any custom extension used by this contract
    pub extension: VoucherExtension,
}
