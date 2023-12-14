use cosmwasm_std::{attr, Attribute, Event, Response};
use neutron_sdk::bindings::msg::NeutronMsg;

pub fn response<A: Into<Attribute>>(
    ty: &str,
    contract_name: &str,
    attrs: impl IntoIterator<Item = A>,
) -> Response<NeutronMsg> {
    Response::new().add_event(Event::new(format!("{}-{}", contract_name, ty)).add_attributes(attrs))
}

pub fn attr_coin(
    key: impl Into<String>,
    amount: impl std::fmt::Display,
    denom: impl std::fmt::Display,
) -> Attribute {
    attr(key, format!("{}{}", amount, denom))
}
