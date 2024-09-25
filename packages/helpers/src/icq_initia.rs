use cosmwasm_std::Binary;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, types::KVKey},
    interchain_queries::{
        helpers::{decode_and_convert, length_prefix},
        types::QueryPayload,
        v047::types::{BALANCES_PREFIX, DELEGATION_KEY, VALIDATORS_KEY},
    },
    NeutronResult,
};
use sha3::{Digest, Sha3_256 as Sha256};

// Looking at https://github.com/initia-labs/initia/blob/dbf293c80719d748bbe5f8142fbaf7a98e37fdcb/x/move/keeper/fungible_asset.go#L19
// we can see that bz returned from k.GetResourceBytes is permanent for every address and denom
// so we can use it as a suffix for every key
const MOVE_VM_STORE_SUFFIX: [u8; 63] = [
    0x2, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0xe, 0x66, 0x75, 0x6e,
    0x67, 0x69, 0x62, 0x6c, 0x65, 0x5f, 0x61, 0x73, 0x73, 0x65, 0x74, 0xd, 0x46, 0x75, 0x6e, 0x67,
    0x69, 0x62, 0x6c, 0x65, 0x53, 0x74, 0x6f, 0x72, 0x65, 0x0,
];

pub fn get_balance_and_delegations_keys(
    delegator: String,
    denom: String,
    validators: Vec<String>,
    store_key: String,
) -> NeutronResult<Vec<KVKey>> {
    let delegator_addr = decode_and_convert(&delegator)?;
    let balance_key = create_account_denom_balance_key(&delegator_addr, denom);
    // Allocate memory for such KV keys as:
    // * validators structures to calculate amount of delegated tokens
    // * delegations structures to get info about delegations itself and balance
    let mut keys: Vec<KVKey> = Vec::with_capacity(validators.len() * 2 + 1);

    // // create KV key to get balance of the delegator
    keys.push(KVKey {
        path: "move".to_string(),
        key: Binary(balance_key.to_vec()),
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
) -> Vec<u8> {
    let mut key: Vec<u8> = vec![0x21]; //VM_STORE_PREFIX
    let addr_key = create_addr_key(addr, denom);
    key.extend_from_slice(&addr_key);
    key.extend_from_slice(&MOVE_VM_STORE_SUFFIX);
    key
}

pub fn create_addr_key<AddrBytes: AsRef<[u8]>, S: AsRef<str>>(
    addr: AddrBytes,
    denom: S,
) -> [u8; 32] {
    let padded_address = pad_with_zeros_to_32_bytes(addr.as_ref());
    let denom_metadata = get_denom_metadata(denom.as_ref().to_string());
    // hash sha256 (padded_address + denom_metadata + 0xFC)
    let mut hasher = Sha256::new();
    hasher.update(padded_address);
    hasher.update(denom_metadata);
    hasher.update([0xFC]);
    let result = hasher.finalize();
    result.as_slice().try_into().unwrap()
}

pub fn create_account_balances_prefix<AddrBytes: AsRef<[u8]>>(
    addr: AddrBytes,
) -> NeutronResult<Vec<u8>> {
    let mut prefix: Vec<u8> = vec![BALANCES_PREFIX];
    prefix.extend_from_slice(length_prefix(addr)?.as_slice());

    Ok(prefix)
}

fn get_denom_metadata(denom: String) -> [u8; 32] {
    if !denom.starts_with("move/") {
        panic!("Denom must start with 'move/'");
    }
    let denom = hex::decode(&denom[5..]).unwrap();
    denom
        .as_slice()
        .try_into()
        .unwrap_or_else(|_| panic!("Denom must be 32 bytes long: {:?} - {}", denom, denom.len()))
}

fn pad_with_zeros_to_32_bytes(data: &[u8]) -> [u8; 32] {
    let mut padded_data = [0u8; 32];
    let len = data.len();
    assert!(len <= 32, "Data is too long to pad");
    padded_data[..len].copy_from_slice(&data[..len]);
    padded_data
}
