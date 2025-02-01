use crate::contract::sudo;
use crate::msg::FungibleTokenPacketData;
use crate::state::FAILED_TRANSFERS;
use cosmwasm_std::{testing::mock_env, to_json_binary};
use drop_helpers::testing::mock_dependencies;
use neutron_sdk::sudo::msg::{RequestPacket, RequestPacketTimeoutHeight};

#[test]
fn call_sudo_error() {
    let mut deps = mock_dependencies(&[]);
    let _ = sudo(
        deps.as_mut(),
        mock_env(),
        neutron_sdk::sudo::msg::TransferSudoMsg::Error {
            request: RequestPacket {
                sequence: Some(0),
                source_port: Some("source_port".to_string()),
                source_channel: Some("source_channel".to_string()),
                destination_port: Some("destination_port".to_string()),
                destination_channel: Some("destination_channel".to_string()),
                data: Some(
                    to_json_binary(&FungibleTokenPacketData {
                        denom: "denom".to_string(),
                        amount: "100".to_string(),
                        sender: "sender".to_string(),
                        receiver: "receiver".to_string(),
                        memo: None,
                    })
                    .unwrap(),
                ),
                timeout_height: Some(RequestPacketTimeoutHeight {
                    revision_height: Some(0),
                    revision_number: Some(0),
                }),
                timeout_timestamp: Some(0),
            },
            details: "sudo-error".to_string(),
        },
    )
    .unwrap();
    println!(
        "{:?}",
        FAILED_TRANSFERS
            .load(&deps.storage, "receiver".to_string())
            .unwrap()
    );
}
