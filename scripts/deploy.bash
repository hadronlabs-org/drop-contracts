#!/usr/bin/env bash

NEUTRON_RPC="tcp://0.0.0.0:26657"
NEUTRON_HOME="../neutron/data/test-1"
NEUTRON_CHAIN_ID="test-1"
TARGET_CHAIN_ID="test-2"
DEPLOY_WALLET="demowallet1"
KEYRING_BACKEND="test"
GAS_PRICES="0.005"
MIN_NTRN_REQUIRED="10"
# TODO: can we obtain this automatically?
TARGET_SDK_VERSION="0.47.10"
TARGET_BASE_DENOM="uatom"
NEUTRON_SIDE_TRANSFER_CHANNEL_ID="channel-0"
IBC_REGISTER_FEE="1000000"
HERMES_CONFIG="../neutron/network/hermes/config.toml"
ARTIFACTS_DIR="../artifacts"

source ./utils.bash

main() {
  set -euo pipefail
  IFS=$'\n\t'

  pre_deploy_check_balance
  pre_deploy_check_ibc_connection
  deploy_wasm_code
  deploy_factory
  setup_staker_ica
  setup_puppeteer_ica
  deploy_pump
  setup_pump_ica

  echo
  echo   "DEPLOY SUCCEDED"
  echo
  printf 'export FACTORY_ADDRESS="%s"\n' "$factory_address"
  printf 'export IBC_DENOM="%s"\n' "$uatom_on_neutron_denom"
  echo
  echo   "[[chains]]"
  printf 'id = "%s"\n' "$NEUTRON_CHAIN_ID"
  echo   "[chains.packet_filter]"
  echo   "list = ["
  echo   "  ['$puppeteer_ica_port', '$puppeteer_ica_channel'],"
  echo   "  ['$pump_ica_port', '$pump_ica_channel'],"
  echo   "  ['$staker_ica_port', '$staker_ica_channel']"
  echo   "]"
  echo
  echo   "[[chains]]"
  printf 'id = "%s"\n' "$TARGET_CHAIN_ID"
  echo   "[chains.packet_filter]"
  echo   "list = ["
  echo   "  ['icahost', '$puppeteer_ica_counterparty_channel'],"
  echo   "  ['icahost', '$pump_ica_counterparty_channel'],"
  echo   "  ['icahost', '$staker_ica_counterparty_channel']"
  echo   "]"
 
}

setup_staker_ica() {
  register_staker_ica

  staker_ica_counterparty_channel="$(hermes --config "$HERMES_CONFIG" tx chan-open-try \
    --dst-chain "$TARGET_CHAIN_ID" --src-chain "$NEUTRON_CHAIN_ID"                   \
    --dst-connection "$target_side_connection_id"                                    \
    --dst-port "icahost" --src-port "$staker_ica_port"                               \
    --src-channel "$staker_ica_channel"                                              \
      | tr -d ' \n' | sed -rn 's/.*,channel_id:Some\(ChannelId\("(channel-[0-9]+)".*/\1/p')"
  echo "[OK] Staker ICA counterparty configuration: icahost/$staker_ica_counterparty_channel"

  hermes --config "$HERMES_CONFIG" tx chan-open-ack                                      \
    --dst-chain "$NEUTRON_CHAIN_ID" --src-chain "$TARGET_CHAIN_ID"                       \
    --dst-connection "$neutron_side_connection_id"                                       \
    --dst-port "$staker_ica_port" --src-port "icahost"                                   \
    --dst-channel "$staker_ica_channel" --src-channel "$staker_ica_counterparty_channel" \
      | grep "SUCCESS" >/dev/null
  echo "[OK] Submitted IBC channel open ACK"

  hermes --config "$HERMES_CONFIG" tx chan-open-confirm                                  \
    --dst-chain "$TARGET_CHAIN_ID" --src-chain "$NEUTRON_CHAIN_ID"                       \
    --dst-connection "$target_side_connection_id"                                        \
    --dst-port "icahost" --src-port "$staker_ica_port"                                   \
    --dst-channel "$staker_ica_counterparty_channel" --src-channel "$staker_ica_channel" \
      | grep "SUCCESS" >/dev/null
  echo "[OK] Submitted IBC channel open CONFIRM"

  staker_ica_address="$(neutrond query wasm contract-state smart "$staker_address" '{"ica":{}}' "${nq[@]}" \
    | jq -r '.data.registered.ica_address')"
  echo "[OK] Staker ICA address: $staker_ica_address"
}

setup_puppeteer_ica() {
  register_puppeteer_ica

  puppeteer_ica_counterparty_channel="$(hermes --config "$HERMES_CONFIG" tx chan-open-try \
    --dst-chain "$TARGET_CHAIN_ID" --src-chain "$NEUTRON_CHAIN_ID"                        \
    --dst-connection "$target_side_connection_id"                                         \
    --dst-port "icahost" --src-port "$puppeteer_ica_port"                                 \
    --src-channel "$puppeteer_ica_channel"                                                \
      | tr -d ' \n' | sed -rn 's/.*,channel_id:Some\(ChannelId\("(channel-[0-9]+)".*/\1/p')"
  echo "[OK] Puppeteer ICA counterparty configuration: icahost/$puppeteer_ica_counterparty_channel"

  hermes --config "$HERMES_CONFIG" tx chan-open-ack                                            \
    --dst-chain "$NEUTRON_CHAIN_ID" --src-chain "$TARGET_CHAIN_ID"                             \
    --dst-connection "$neutron_side_connection_id"                                             \
    --dst-port "$puppeteer_ica_port" --src-port "icahost"                                      \
    --dst-channel "$puppeteer_ica_channel" --src-channel "$puppeteer_ica_counterparty_channel" \
      | grep "SUCCESS" >/dev/null
  echo "[OK] Submitted IBC channel open ACK"

  hermes --config "$HERMES_CONFIG" tx chan-open-confirm                                        \
    --dst-chain "$TARGET_CHAIN_ID" --src-chain "$NEUTRON_CHAIN_ID"                             \
    --dst-connection "$target_side_connection_id"                                              \
    --dst-port "icahost" --src-port "$puppeteer_ica_port"                                      \
    --dst-channel "$puppeteer_ica_counterparty_channel" --src-channel "$puppeteer_ica_channel" \
      | grep "SUCCESS" >/dev/null
  echo "[OK] Submitted IBC channel open CONFIRM"

  puppeteer_ica_address="$(neutrond query wasm contract-state smart "$puppeteer_address" '{"ica":{}}' "${nq[@]}" \
    | jq -r '.data.registered.ica_address')"
  echo "[OK] Puppeteer ICA address: $puppeteer_ica_address"
}


setup_pump_ica() {
  register_pump_ica

  pump_ica_counterparty_channel="$(hermes --config "$HERMES_CONFIG" tx chan-open-try \
    --dst-chain "$TARGET_CHAIN_ID" --src-chain "$NEUTRON_CHAIN_ID"                   \
    --dst-connection "$target_side_connection_id"                                    \
    --dst-port "icahost" --src-port "$pump_ica_port"                                 \
    --src-channel "$pump_ica_channel"                                                \
      | tr -d ' \n' | sed -rn 's/.*,channel_id:Some\(ChannelId\("(channel-[0-9]+)".*/\1/p')"
  echo "[OK] Pump ICA counterparty configuration: icahost/$pump_ica_counterparty_channel"

  hermes --config "$HERMES_CONFIG" tx chan-open-ack                                  \
    --dst-chain "$NEUTRON_CHAIN_ID" --src-chain "$TARGET_CHAIN_ID"                   \
    --dst-connection "$neutron_side_connection_id"                                   \
    --dst-port "$pump_ica_port" --src-port "icahost"                                 \
    --dst-channel "$pump_ica_channel" --src-channel "$pump_ica_counterparty_channel" \
      | grep "SUCCESS" >/dev/null
  echo "[OK] Submitted IBC channel open ACK"

  hermes --config "$HERMES_CONFIG" tx chan-open-confirm                              \
    --dst-chain "$TARGET_CHAIN_ID" --src-chain "$NEUTRON_CHAIN_ID"                   \
    --dst-connection "$target_side_connection_id"                                    \
    --dst-port "icahost" --src-port "$pump_ica_port"                                 \
    --dst-channel "$pump_ica_counterparty_channel" --src-channel "$pump_ica_channel" \
      | grep "SUCCESS" >/dev/null
  echo "[OK] Submitted IBC channel open CONFIRM"

  pump_ica_address="$(neutrond query wasm contract-state smart "$pump_address" '{"ica":{}}' "${nq[@]}" \
    | jq -r '.data.registered.ica_address')"
  echo "[OK] Pump ICA address: $pump_ica_address"

  msg='{
    "update_config":{
      "core":{
        "pump_ica_address":"'"$pump_ica_address"'"
      }
    }
  }'
  neutrond tx wasm execute "$factory_address" "$msg" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx | assert_success
  echo "[OK] Add pump ICA address to Core config"
}

# note for myself: don't write programs in bash
exec 3>&1
error_output="$(main 2>&1 1>&3)"
exit_code=$?
exec 3>&-

if [[ ! $exit_code -eq 0 ]]; then
  echo
  echo "DEPLOY FAILED WITH CODE $exit_code"
  echo "Error output:"
  echo "$error_output"
fi

exit $exit_code
