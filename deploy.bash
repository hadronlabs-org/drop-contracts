#!/usr/bin/env bash

NEUTRON_RPC="tcp://0.0.0.0:26657"
NEUTRON_HOME="../neutron/data/test-1"
NEUTRON_CHAIN_ID="test-1"
TARGET_CHAIN_ID="test-2"
DEPLOY_WALLET="demowallet1"
GAS_PRICES="0.005"
MIN_NTRN_REQUIRED="10"
# TODO: can we obtain this automatically?
TARGET_SDK_VERSION="0.47.10"
TARGET_BASE_DENOM="uatom"
NEUTRON_SIDE_TRANSFER_CHANNEL_ID="channel-0"
IBC_ACK_FEE="10000"
IBC_TIMEOUT_FEE="$IBC_ACK_FEE"
IBC_REGISTER_FEE="1000000"
HERMES_CONFIG="../neutron/network/hermes/config.toml"

declare -a ntx=(
  "--home" "$NEUTRON_HOME"
  "--keyring-backend" "test"
  "--broadcast-mode" "sync"
  "--gas" "auto"
  "--gas-adjustment" "1.5"
  "--gas-prices" "${GAS_PRICES}untrn"
  "--node" "$NEUTRON_RPC"
  "--chain-id" "$NEUTRON_CHAIN_ID"
  "--output" "json"
  "-y"
)

declare -a nq=(
  "--node" "$NEUTRON_RPC"
  "--output" "json"
)

die() {
  echo "$1" >&2
  exit 1
}

wait_ntx() {
  wait_tx "neutrond" "nq"
}

wait_tx() {
  local aname
  local q
  local txhash
  local tx
  local code
  local attempts
  aname="$2[@]"
  q=("${!aname}")
  tx="$(cat /dev/stdin)"
  code="$(echo "$tx" | jq -r '.code')"
  if [[ ! $code -eq 0 ]]; then
    die "Tx failed with code $code and message: $(echo "$tx" | jq -r '.raw_log')"
  fi
  txhash="$(echo "$tx" | jq -r '.txhash')"
  if [[ ! ${#txhash} -eq 64 ]]; then
    die "No txhash found in: $tx"
  fi
  ((attempts=200))
  while ! "$1" query tx --type=hash "$txhash" "${q[@]}" 2>/dev/null; do
    ((attempts-=1)) || {
      die "tx $txhash still not included in block"
    }
    sleep 0.1
  done
}

select_attr() {
  printf '.logs[0].events[] | select(.type == "%s").attributes[] | select(.key == "%s").value' "$1" "$2"
}

assert_success() {
  local tx_status
  local code
  tx_status="$(cat /dev/stdin)"
  code="$(echo "$tx_status" | jq -r '.code')"
  if [[ $code -ne 0 ]]; then
    echo "[FAIL] tx failed:"
    echo "$tx_status" | jq
    exit 1
  fi
}

store_code() {
  local res
  res="$(neutrond tx wasm store "artifacts/drop_$1.wasm" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  declare -g "$1_code_id=$(echo "$res" | jq -r "$(select_attr "store_code" "code_id")")"
}

main() {
  set -euo pipefail
  IFS=$'\n\t'

  pre_deploy_check_balance
  pre_deploy_check_ibc_connection
  deploy_wasm_code
  deploy_factory
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
  echo   "  ['icahost', '$puppeteer_ica_counterparty_channel'],"
  echo   "  ['icahost', '$pump_ica_counterparty_channel']"
  echo   "]"
  echo
  echo   "[[chains]]"
  printf 'id = "%s"\n' "$TARGET_CHAIN_ID"
  echo   "[chains.packet_filter]"
  echo   "list = ["
  echo   "  ['$puppeteer_ica_port', '$puppeteer_ica_channel'],"
  echo   "  ['$pump_ica_port', '$pump_ica_channel']"
  echo   "]"
}

pre_deploy_check_balance() {
  deploy_wallet="$(neutrond keys show "$DEPLOY_WALLET" --home "$NEUTRON_HOME" --keyring-backend test --output json | jq -r '.address')"
  untrn_balance="$(neutrond query bank balances --denom=untrn "$deploy_wallet" "${nq[@]}" | jq -r '.amount')"
  ntrn_balance="$(echo "$untrn_balance / (10^6)" | bc)"
  ntrn_balance_decimals="$(echo "$untrn_balance % (10^6)" | bc)"
  ntrn_balance_human="${ntrn_balance}.${ntrn_balance_decimals}NTRN"
  if [[ $ntrn_balance -lt $MIN_NTRN_REQUIRED ]]; then
    die "$DEPLOY_WALLET [$deploy_wallet] only has $ntrn_balance_human while at least ${MIN_NTRN_REQUIRED}NTRN are required"
  fi
  echo "[OK] $DEPLOY_WALLET has sufficient balance of $ntrn_balance_human"
}

pre_deploy_check_ibc_connection() {
  channel_info="$(neutrond query ibc channel end transfer "$NEUTRON_SIDE_TRANSFER_CHANNEL_ID" "${nq[@]}")"
  connection_hops="$(echo "$channel_info" | jq -r '.channel.connection_hops | length')"
  if [[ ! $connection_hops -eq 1 ]]; then
    die "$NEUTRON_SIDE_TRANSFER_CHANNEL_ID has unsupported amount of connection hops: $connection_hops"
  fi
  target_side_transfer_channel_id="$(echo "$channel_info" | jq -r '.channel.counterparty.channel_id')"
  neutron_side_connection_id="$(echo "$channel_info" | jq -r '.channel.connection_hops[0]')"
  target_side_connection_id="$(neutrond query ibc connection end "$neutron_side_connection_id" "${nq[@]}" \
    | jq -r '.connection.counterparty.connection_id')"
  echo "[OK] Neutron side transfer channel ID: $NEUTRON_SIDE_TRANSFER_CHANNEL_ID"
  echo "[OK] Target  side transfer channel ID: $target_side_transfer_channel_id"
  echo "[OK] Neutron side       connection ID: $neutron_side_connection_id"
  echo "[OK] Target  side       connection ID: $target_side_connection_id"
}

deploy_wasm_code() {
  for contract in factory core distribution puppeteer rewards_manager strategy token staker validators_set withdrawal_manager withdrawal_voucher pump; do
      store_code "$contract"
      code_id="${contract}_code_id"
      printf '[OK] %-24s code ID: %s\n' "$contract" "${!code_id}"
  done
}

# code IDs are assigned using dynamic variable names, shellcheck's mind cannot comprehend that
# shellcheck disable=SC2154
deploy_factory() {
  # TODO: calculate unbond batch switch time and unbonding period using params queried from the network
  uatom_on_neutron_denom="ibc/$(printf 'transfer/%s/%s' "$NEUTRON_SIDE_TRANSFER_CHANNEL_ID" "$TARGET_BASE_DENOM" \
    | sha256sum - | awk '{print $1}' | tr '[:lower:]' '[:upper:]')"
  echo "[OK] IBC denom of $TARGET_BASE_DENOM on Neutron is $uatom_on_neutron_denom"
  msg='{
    "sdk_version":"'"$TARGET_SDK_VERSION"'",
    "code_ids": {
      "core_code_id":'"$core_code_id"',
      "token_code_id":'"$token_code_id"',
      "withdrawal_voucher_code_id":'"$withdrawal_voucher_code_id"',
      "withdrawal_manager_code_id":'"$withdrawal_manager_code_id"',
      "strategy_code_id":'"$strategy_code_id"',
      "distribution_code_id":'"$distribution_code_id"',
      "validators_set_code_id":'"$validators_set_code_id"',
      "puppeteer_code_id":'"$puppeteer_code_id"',
      "staker_code_id":'"$staker_code_id"',
      "rewards_manager_code_id":'"$rewards_manager_code_id"'
    },
    "remote_opts":{
      "connection_id":"'"$neutron_side_connection_id"'",
      "transfer_channel_id":"'"$NEUTRON_SIDE_TRANSFER_CHANNEL_ID"'",
      "port_id":"transfer",
      "denom":"'"$TARGET_BASE_DENOM"'",
      "update_period":100,
      "ibc_fees":{
        "timeout_fee":"'"$IBC_TIMEOUT_FEE"'",
        "ack_fee":"'"$IBC_ACK_FEE"'",
        "recv_fee":"0",
        "register_fee":"'"$IBC_REGISTER_FEE"'"
      }
    },
    "salt":"salt",
    "subdenom":"drop",
    "token_metadata":{
      "description":"Drop token",
      "display":"drop",
      "exponent":6,
      "name":"Drop liquid staking token",
      "symbol":"DROP"
    },
    "base_denom":"'"$uatom_on_neutron_denom"'",
    "core_params":{
      "idle_min_interval":60,
      "puppeteer_timeout":120,
      "unbond_batch_switch_time":259200,
      "unbonding_safe_period":3600,
      "unbonding_period":1814400,
      "lsm_redeem_threshold":2,
      "lsm_min_bond_amount":"1",
      "lsm_redeem_max_interval":60000,
      "bond_limit":"0",
      "min_stake_amount":"2"
    },
    "staker_params":{
      "min_stake_amount":"10000",
      "min_ibc_transfer":"10000"
    }
  }'
  factory_address="$(neutrond tx wasm instantiate "$factory_code_id" "$msg" \
    --label "drop-staking-factory"                                          \
    --admin "$deploy_wallet"                                                \
    --from "$DEPLOY_WALLET" "${ntx[@]}"                                     \
      | wait_ntx | jq -r "$(select_attr "wasm-crates.io:drop-staking__drop-factory-instantiate" "_contract_address")")"
  echo "[OK] Factory address: $factory_address"
  puppeteer_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.puppeteer_contract')"
  withdrawal_manager_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.withdrawal_manager_contract')"
  echo "[OK] Puppeteer contract: $puppeteer_address"
  echo "[OK] Withdrawal manager contract: $withdrawal_manager_address"
  msg='{
    "update_config":{
      "puppeteer_fees":{
        "timeout_fee":"'"$IBC_TIMEOUT_FEE"'",
        "ack_fee":"'"$IBC_ACK_FEE"'",
        "recv_fee":"0",
        "register_fee":"'"$IBC_REGISTER_FEE"'"
      }
    }
  }'
  neutrond tx wasm execute "$factory_address" "$msg" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx | assert_success
  echo "[OK] Set puppeteer fees"
}

setup_puppeteer_ica() {
  register_ica_result="$(neutrond tx wasm execute "$puppeteer_address" '{"register_i_c_a":{}}' \
    --amount "${IBC_REGISTER_FEE}untrn" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  puppeteer_ica_port="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "port_id")")"
  puppeteer_ica_channel="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "channel_id")")"
  echo "[OK] Puppeteer ICA configuration: $puppeteer_ica_port/$puppeteer_ica_channel"

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

deploy_pump() {
  msg='{
    "connection_id":"'"$neutron_side_connection_id"'",
    "ibc_fees":{
      "timeout_fee":"'"$IBC_TIMEOUT_FEE"'",
      "ack_fee":"'"$IBC_ACK_FEE"'",
      "recv_fee":"0",
      "register_fee":"'"$IBC_REGISTER_FEE"'"
    },
    "local_denom":"untrn",
    "timeout":{
      "local":360,
      "remote":360
    },
    "dest_address":"'"$withdrawal_manager_address"'",
    "dest_port":"transfer",
    "dest_channel":"'"$target_side_transfer_channel_id"'",
    "refundee":"'"$deploy_wallet"'",
    "owner":"'"$deploy_wallet"'"
  }'
  # pump code ID is assigned using dynamic variable name, shellcheck's mind cannot comprehend that
  # shellcheck disable=SC2154
  pump_address="$(neutrond tx wasm instantiate "$pump_code_id" "$msg"                \
    --label "drop-pump" --admin "$deploy_wallet" --from "$DEPLOY_WALLET" "${ntx[@]}" \
      | wait_ntx | jq -r "$(select_attr "instantiate" "_contract_address")")"
  echo "[OK] Pump address: $pump_address"
}

setup_pump_ica() {
  register_ica_result="$(neutrond tx wasm execute "$pump_address" '{"register_i_c_a":{}}' \
    --amount "${IBC_REGISTER_FEE}untrn" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  pump_ica_port="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "port_id")")"
  pump_ica_channel="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "channel_id")")"
  echo "[OK] Pump ICA configuration: $pump_ica_port/$pump_ica_channel"

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
