use cosmwasm_std::{Addr, Deps, StdError, StdResult};

const DEFAULT_MAX_ADDRESSES: usize = 16;

pub fn validate_addresses(
    deps: Deps,
    input: &[impl AsRef<str> + Ord],
    max_addresses: Option<usize>,
) -> StdResult<Vec<Addr>> {
    let max_addresses = max_addresses.unwrap_or(DEFAULT_MAX_ADDRESSES);
    if input.len() > max_addresses {
        return Err(StdError::generic_err(format!(
            "Too many addresses, max: {max_addresses}"
        )));
    }
    let mut input: Vec<&_> = input.iter().collect();
    input.sort();
    input.dedup();
    input
        .iter()
        .map(|addr| deps.api.addr_validate(addr.as_ref()))
        .collect::<StdResult<Vec<_>>>()
}
