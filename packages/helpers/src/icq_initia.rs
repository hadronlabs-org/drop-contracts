use cosmwasm_std::Binary;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, types::KVKey},
    interchain_queries::{
        helpers::{decode_and_convert, length_prefix},
        types::QueryPayload,
        v045::types::BANK_STORE_KEY,
        v047::types::{BALANCES_PREFIX, DELEGATION_KEY, VALIDATORS_KEY},
    },
    NeutronResult,
};

pub fn get_balance_and_delegations_keys(
    delegator: String,
    denom: String,
    validators: Vec<String>,
    store_key: String,
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

    for v in validators {
        let val_addr = decode_and_convert(&v)?;

        // create delegation key to get delegation structure
        keys.push(KVKey {
            path: store_key.to_string(),
            key: Binary(create_delegation_key(&delegator_addr, &val_addr)?),
        });

        // create validator key to get validator structure
        keys.push(KVKey {
            path: store_key.to_string(),
            key: Binary(create_validator_key(&val_addr)?),
        })
    }

    Ok(keys)
}

fn create_validator_key<AddrBytes: AsRef<[u8]>>(
    operator_address: AddrBytes,
) -> NeutronResult<Vec<u8>> {
    let mut key: Vec<u8> = vec![VALIDATORS_KEY];
    key.extend_from_slice(operator_address.as_ref());

    Ok(key)
}

fn create_delegation_key<AddrBytes: AsRef<[u8]>>(
    delegator_address: AddrBytes,
    validator_address: AddrBytes,
) -> NeutronResult<Vec<u8>> {
    let mut delegations_key: Vec<u8> = create_delegations_key(delegator_address)?;
    delegations_key.extend_from_slice(validator_address.as_ref());
    Ok(delegations_key)
}

fn create_delegations_key<AddrBytes: AsRef<[u8]>>(
    delegator_address: AddrBytes,
) -> NeutronResult<Vec<u8>> {
    let mut key: Vec<u8> = vec![DELEGATION_KEY];
    key.extend_from_slice(length_prefix(delegator_address)?.as_slice());
    Ok(key)
}

/// Create a query message to get delegations and balance from a delegator to a list of validators
pub fn new_delegations_and_balance_query_msg(
    connection_id: String,
    delegator: String,
    denom: String,
    validators: Vec<String>,
    update_period: u64,
) -> NeutronResult<NeutronMsg> {
    let keys =
        get_balance_and_delegations_keys(delegator, denom, validators, "mstaking".to_string())?;
    NeutronMsg::register_interchain_query(QueryPayload::KV(keys), connection_id, update_period)
}

pub fn create_account_denom_balance_key<AddrBytes: AsRef<[u8]>, S: AsRef<str>>(
    addr: AddrBytes,
    denom: S,
) -> NeutronResult<Vec<u8>> {
    let mut account_balance_key = create_account_balances_prefix(addr)?;
    account_balance_key.extend_from_slice(denom.as_ref().as_bytes());

    Ok(account_balance_key)
}

pub fn create_account_balances_prefix<AddrBytes: AsRef<[u8]>>(
    addr: AddrBytes,
) -> NeutronResult<Vec<u8>> {
    let mut prefix: Vec<u8> = vec![BALANCES_PREFIX];
    prefix.extend_from_slice(length_prefix(addr)?.as_slice());

    Ok(prefix)
}
