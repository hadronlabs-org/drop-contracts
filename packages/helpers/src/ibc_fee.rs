use cosmwasm_std::{Deps, QueryRequest, StdResult};
use neutron_sdk::{
    bindings::{msg::IbcFee, query::NeutronQuery},
    query::min_ibc_fee::MinIbcFeeResponse,
};

pub fn query_ibc_fee(deps: Deps<NeutronQuery>, denom: &str) -> StdResult<IbcFee> {
    let min_fee: MinIbcFeeResponse = deps
        .querier
        .query(&QueryRequest::Custom(NeutronQuery::MinIbcFee {}))?;

    let mut fee = min_fee.min_fee;
    fee.ack_fee.retain(|fee| fee.denom == denom);
    fee.timeout_fee.retain(|fee| fee.denom == denom);

    Ok(fee)
}
