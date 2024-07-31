#!/usr/bin/env bash

declare -a ntx=(
  "--home" "$NEUTRON_HOME"
  "--keyring-backend" "$KEYRING_BACKEND"
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
  printf '.events[] | select(.type == "%s").attributes[] | select(.key == "%s").value' "$1" "$2"
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
  res="$(neutrond tx wasm store "$ARTIFACTS_DIR/drop_$1.wasm" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  declare -g "$1_code_id=$(echo "$res" | jq -r "$(select_attr "store_code" "code_id")")"
}

deploy_wasm_code() {
  for contract in factory core distribution puppeteer rewards_manager strategy token staker validators_set withdrawal_manager withdrawal_voucher pump splitter; do
      store_code "$contract"
      code_id="${contract}_code_id"
      printf '[OK] %-24s code ID: %s\n' "$contract" "${!code_id}"
  done
}

pre_deploy_check_balance() {
  deploy_wallet="$(neutrond keys show "$DEPLOY_WALLET" \
    --home "$NEUTRON_HOME"                             \
    --keyring-backend "$KEYRING_BACKEND"               \
    --output json | jq -r '.address')"
  untrn_balance="$(neutrond query bank balance "$deploy_wallet" untrn "${nq[@]}" | jq -r '.balance.amount')"
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

pre_deploy_check_code_ids() {
  for contract in factory core distribution puppeteer rewards_manager strategy token staker validators_set withdrawal_manager withdrawal_voucher pump splitter; do
    code_id="${contract}_code_id"
    set +u
    if [[ -z "${!code_id}" ]]; then
      die "Code ID for $contract is not set"
    fi
    set -u
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
    "local_denom":"untrn",
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
      "rewards_manager_code_id":'"$rewards_manager_code_id"',
      "splitter_code_id": '"$splitter_code_id"',
      "rewards_pump_code_id": '"$pump_code_id"'
    },
    "remote_opts":{
      "connection_id":"'"$neutron_side_connection_id"'",
      "transfer_channel_id":"'"$NEUTRON_SIDE_TRANSFER_CHANNEL_ID"'",
      "port_id":"'"$NEUTRON_SIDE_PORT_ID"'",
      "denom":"'"$TARGET_BASE_DENOM"'",
      "update_period":'$ICQ_UPDATE_PERIOD',
      "timeout":{
        "local":'$TIMEOUT_LOCAL',
        "remote":'$TIMEOUT_REMOTE'
      }
    },
    "salt":"'"$SALT"'",
    "subdenom":"'"$SUBDENOM"'",
    "token_metadata":{
      "description":"'"$TOKEN_METADATA_DESCRIPTION"'",
      "display":"'"$TOKEN_METADATA_DISPLAY"'",
      "exponent":'$TOKEN_METADATA_EXPONENT',
      "name":"'"$TOKEN_METADATA_NAME"'",
      "symbol":"'"$TOKEN_METADATA_SYMBOL"'"
    },
    "base_denom":"'"$uatom_on_neutron_denom"'",
    "core_params":{
      "idle_min_interval":'$CORE_PARAMS_IDLE_MIN_INTERVAL',
      "unbond_batch_switch_time":'"$UNBOND_BATCH_SWITCH_TIME"',
      "unbonding_safe_period":'"$UNBONDING_SAFE_PERIOD"',
      "unbonding_period":'"$UNBONDING_PERIOD"',
      "lsm_redeem_threshold":'$CORE_PARAMS_LSM_REDEEM_THRESHOLD',
      "lsm_min_bond_amount":"'"$CORE_PARAMS_LSM_MIN_BOND_AMOUNT"'",
      "lsm_redeem_max_interval":'$CORE_PARAMS_LSM_REDEEM_MAX_INTERVAL',
      "bond_limit":"'"$CORE_PARAMS_BOND_LIMIT"'",
      "min_stake_amount":"'"$CORE_PARAMS_MIN_STAKE_AMOUNT"'",
      "icq_update_delay": '$CORE_PARAMS_ICQ_UPDATE_DELAY'
    },
    "staker_params":{
      "min_stake_amount":"'"$STAKER_PARAMS_MIN_STAKE_AMOUNT"'",
      "min_ibc_transfer":"'"$STAKER_PARAMS_MIN_IBC_TRANSFER"'"
    }
  }'
  factory_address="$(neutrond tx wasm instantiate "$factory_code_id" "$msg" \
    --label "drop-staking-factory"                                          \
    --admin "$deploy_wallet"                                                \
    --from "$DEPLOY_WALLET" "${ntx[@]}"                                     \
      | wait_ntx | jq -r "$(select_attr "wasm-crates.io:drop-staking__drop-factory-instantiate" "_contract_address")")"
  echo "[OK] Factory address: $factory_address"
  staker_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.staker_contract')"
  splitter_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.splitter_contract')"
  rewards_pump_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.rewards_pump_contract')"
  splitter_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.rewards_pump_contract')"
  puppeteer_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.puppeteer_contract')"
  withdrawal_manager_address="$(neutrond query wasm contract-state smart "$factory_address" '{"state":{}}' "${nq[@]}" \
    | jq -r '.data.withdrawal_manager_contract')"
  echo "[OK] Staker contract: $staker_address"
  echo "[OK] Puppeteer contract: $puppeteer_address"
  echo "[OK] Withdrawal manager contract: $withdrawal_manager_address"
}

get_ibc_register_fee() {
  neutrond query interchaintxs params "${nq[@]}" | jq -r '.params.register_fee[] | select(.denom=="untrn") | .amount'
}

register_staker_ica() {
  register_ica_result="$(neutrond tx wasm execute "$staker_address" '{"register_i_c_a":{}}' \
    --amount "$(get_ibc_register_fee)untrn" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  staker_ica_port="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "port_id")")"
  staker_ica_channel="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "channel_id")")"
  echo "[OK] Staker ICA configuration: $staker_ica_port/$staker_ica_channel"
}

register_rewards_pump_ica() {
  register_ica_result="$(neutrond tx wasm execute "$rewards_pump_address" '{"register_i_c_a":{}}' \
    --amount "$(get_ibc_register_fee)untrn" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  rewards_pump_ica_port="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "port_id")")"
  rewards_pump_ica_channel="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "channel_id")")"
  echo "[OK] Rewards pump ICA configuration: $rewards_pump_ica_port/$rewards_pump_ica_channel"
}

register_puppeteer_ica() {
  register_ica_result="$(neutrond tx wasm execute "$puppeteer_address" '{"register_i_c_a":{}}' \
    --amount "$(get_ibc_register_fee)untrn" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  puppeteer_ica_port="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "port_id")")"
  puppeteer_ica_channel="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "channel_id")")"
  echo "[OK] Puppeteer ICA configuration: $puppeteer_ica_port/$puppeteer_ica_channel"
}

register_pump_ica() {
  register_ica_result="$(neutrond tx wasm execute "$pump_address" '{"register_i_c_a":{}}' \
    --amount "$(get_ibc_register_fee)untrn" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx)"
  pump_ica_port="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "port_id")")"
  pump_ica_channel="$(echo "$register_ica_result" | jq -r "$(select_attr "channel_open_init" "channel_id")")"
  echo "[OK] Pump ICA configuration: $pump_ica_port/$pump_ica_channel"
}

print_hermes_command() {
    local ICA_PORT="$1"
    local ICA_CHANNEL="$2"

    echo ""
    echo "hermes tx chan-open-try \\"
    echo "--dst-chain \"$TARGET_CHAIN_ID\" --src-chain \"$NEUTRON_CHAIN_ID\" \\"
    echo "--dst-connection \"$target_side_connection_id\" \\"
    echo "--dst-port \"icahost\" --src-port \"$ICA_PORT\" \\"
    echo "--src-channel \"$ICA_CHANNEL\""
    echo ""

    echo ""
    echo "hermes tx chan-open-ack \\"
    echo "--dst-chain \"$NEUTRON_CHAIN_ID\" --src-chain \"$TARGET_CHAIN_ID\" \\"
    echo "--dst-connection \"$neutron_side_connection_id\" \\"
    echo "--dst-port \"$ICA_PORT\" --src-port "icahost" \\"
    echo "--dst-channel \"$ICA_CHANNEL\" --src-channel \"<ICA COUNTERPARTY CHANNEL FROM chan-open-try COMMAND>\""
    echo ""

    echo ""
    echo "hermes tx chan-open-confirm \\"
    echo "--dst-chain \"$TARGET_CHAIN_ID\" --src-chain \"$NEUTRON_CHAIN_ID\" \\"
    echo "--dst-connection \"$target_side_connection_id\" \\"
    echo "--dst-port \"icahost\" --src-port \"$ICA_PORT\" \\"
    echo "--dst-channel \"<CHANNEL FROM chan-open-try COMMAND>\" --src-channel \"$ICA_CHANNEL\""
    echo ""
}

wait_ica_address() {
  local contract_name="$1"
  local contract_address="$2"

  while true; do
    ica_address="$(neutrond query wasm contract-state smart "$contract_address" '{"ica":{}}' "${nq[@]}" \
      | jq -r 'try (.data.registered.ica_address) catch ""')"
    if [[ -n "$ica_address" ]]; then
      echo "[OK] $contract_name ICA address: $ica_address"
      declare -g "${contract_name}_ica_address=$ica_address"
      break
    fi
    sleep 5
  done  
}

deploy_pump() {
  msg='{
    "connection_id":"'"$neutron_side_connection_id"'",
    "local_denom":"untrn",
    "timeout":{
      "local":'$TIMEOUT_LOCAL',
      "remote":'$TIMEOUT_REMOTE'
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

factory_admin_execute() {
  local factory_address="$1"
  local sub_msg="$2"
  local amount="${3:-0untrn}"

  local msg='{
    "admin_execute": {
      "msgs":[
        '$sub_msg'
      ]
    }
  }'

  neutrond tx wasm execute "$factory_address" "$msg" --amount "$amount" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx | assert_success
}

factory_proxy_execute() {
  local factory_address="$1"
  local sub_msg="$2"
  local amount="${3:-0untrn}"

  local msg='{
    "proxy": '$sub_msg'
  }'

  echo "$msg" | jq '.'

  neutrond tx wasm execute "$factory_address" "$msg" --amount "$amount" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx | assert_success
}

get_counterparty_channel_id() {
  local ica_port="$1"
  local ica_channel="$2"

  local counterparty_channel_id=$(neutrond q ibc channel end $ica_port $ica_channel "${nq[@]}" | jq -r '.channel.counterparty.channel_id')
  echo "$counterparty_channel_id"
}
