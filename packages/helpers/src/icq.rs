use cosmwasm_std::Binary;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, types::KVKey},
    interchain_queries::{
        helpers::decode_and_convert,
        types::QueryPayload,
        v045::{
            helpers::{
                create_account_denom_balance_key, create_delegation_key, create_params_store_key,
                create_validator_key,
            },
            types::{BANK_STORE_KEY, KEY_BOND_DENOM, PARAMS_STORE_KEY, STAKING_STORE_KEY},
        },
        v047::types::STAKING_PARAMS_KEY,
    },
    NeutronResult,
};

use crate::version::version_to_u32;

pub fn new_multiple_balances_query_msg(
    connection_id: String,
    address: String,
    denoms: Vec<String>,
    update_period: u64,
) -> NeutronResult<NeutronMsg> {
    let keys = get_multiple_balances_keys(address, denoms)?;
    NeutronMsg::register_interchain_query(QueryPayload::KV(keys), connection_id, update_period)
}

pub fn update_multiple_balances_query_msg(
    query_id: u64,
    address: String,
    denoms: Vec<String>,
) -> NeutronResult<NeutronMsg> {
    let keys = get_multiple_balances_keys(address, denoms)?;
    NeutronMsg::update_interchain_query(query_id, Some(keys), None, None)
}

/// Create a query message to get delegations and balance from a delegator to a list of validators
pub fn new_delegations_and_balance_query_msg(
    connection_id: String,
    delegator: String,
    denom: String,
    validators: Vec<String>,
    update_period: u64,
    sdk_version: &str,
) -> NeutronResult<NeutronMsg> {
    let keys = get_balance_and_delegations_keys(delegator, denom, validators, sdk_version)?;
    NeutronMsg::register_interchain_query(QueryPayload::KV(keys), connection_id, update_period)
}

pub fn update_balance_and_delegations_query_msg(
    query_id: u64,
    delegator: String,
    denom: String,
    validators: Vec<String>,
    sdk_version: &str,
) -> NeutronResult<NeutronMsg> {
    let keys = get_balance_and_delegations_keys(delegator, denom, validators, sdk_version)?;
    NeutronMsg::update_interchain_query(query_id, Some(keys), None, None)
}

pub fn get_multiple_balances_keys(
    address: String,
    denoms: Vec<String>,
) -> NeutronResult<Vec<KVKey>> {
    let addr = decode_and_convert(&address)?;
    let mut keys: Vec<KVKey> = Vec::with_capacity(denoms.len());
    for denom in denoms {
        let balance_key = create_account_denom_balance_key(&addr, denom)?;
        keys.push(KVKey {
            path: BANK_STORE_KEY.to_string(),
            key: Binary(balance_key),
        });
    }
    Ok(keys)
}

pub fn get_balance_and_delegations_keys(
    delegator: String,
    denom: String,
    validators: Vec<String>,
    sdk_version: &str,
) -> NeutronResult<Vec<KVKey>> {
    let delegator_addr = decode_and_convert(&delegator)?;
    let balance_key = create_account_denom_balance_key(&delegator_addr, denom)?;
    // Allocate memory for such KV keys as:
    // * staking module params to get staking denomination
    // * validators structures to calculate amount of delegated tokens
    // * delegations structures to get info about delegations itself and balance
    let mut keys: Vec<KVKey> = Vec::with_capacity(validators.len() * 2 + 1);

    // // create KV key to get balance of the delegator
    keys.push(KVKey {
        path: BANK_STORE_KEY.to_string(),
        key: Binary(balance_key),
    });

    // create KV key to get BondDenom from staking module params
    if version_to_u32(sdk_version)? < version_to_u32("0.47.0")? {
        keys.push(KVKey {
            path: PARAMS_STORE_KEY.to_string(),
            key: Binary(create_params_store_key(STAKING_STORE_KEY, KEY_BOND_DENOM)),
        });
    } else {
        keys.push(KVKey {
            path: STAKING_STORE_KEY.to_string(),
            key: Binary(vec![STAKING_PARAMS_KEY]),
        });
    }

    for v in validators {
        let val_addr = decode_and_convert(&v)?;

        // create delegation key to get delegation structure
        keys.push(KVKey {
            path: STAKING_STORE_KEY.to_string(),
            key: Binary(create_delegation_key(&delegator_addr, &val_addr)?),
        });

        // create validator key to get validator structure
        keys.push(KVKey {
            path: STAKING_STORE_KEY.to_string(),
            key: Binary(create_validator_key(&val_addr)?),
        })
    }

    Ok(keys)
}
