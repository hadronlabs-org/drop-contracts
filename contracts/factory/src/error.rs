use cosmwasm_std::Instantiate2AddressError;

pub enum ContractError {
    Instantiate2Address {
        err: Instantiate2AddressError,
        code_id: u64,
    },
    Unauthorized {},
    Unimplemented {},
    Unknown {},
}
