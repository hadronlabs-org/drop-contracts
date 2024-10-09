#!/usr/bin/env bash

NEUTRON_RPC="${NEUTRON_RPC:-tcp://0.0.0.0:26657}"
NEUTRON_HOME="${NEUTRON_HOME:-$HOME/.neutrond}"
NEUTRON_CHAIN_ID="${NEUTRON_CHAIN_ID:-test-1}"
TARGET_CHAIN_ID="${TARGET_CHAIN_ID:-test-2}"
GAS_PRICES="${GAS_PRICES:-0.005}"
KEYRING_BACKEND="${KEYRING_BACKEND:-test}"
DEPLOY_WALLET="${DEPLOY_WALLET:-demowallet1}"
MIN_NTRN_REQUIRED="${MIN_NTRN_REQUIRED:-10}"

TARGET_SDK_VERSION="${TARGET_SDK_VERSION:?Variable should be explicitly specified}"
TARGET_BASE_DENOM="${TARGET_BASE_DENOM:?Variable should be explicitly specified}"
NEUTRON_SIDE_TRANSFER_CHANNEL_ID="${NEUTRON_SIDE_TRANSFER_CHANNEL_ID:?Variable should be explicitly specified}"
INITIAL_VALIDATORS="${INITIAL_VALIDATORS:?Variable should be explicitly specified}"
UNBONDING_PERIOD="${UNBONDING_PERIOD:?Variable should be explicitly specified}"
UNBONDING_SAFE_PERIOD="${UNBONDING_SAFE_PERIOD:?Variable should be explicitly specified}"
UNBOND_BATCH_SWITCH_TIME="${UNBOND_BATCH_SWITCH_TIME:?Variable should be explicitly specified}"
TIMEOUT_LOCAL="${TIMEOUT_LOCAL:?Variable should be explicitly specified}"
TIMEOUT_REMOTE="${TIMEOUT_REMOTE:?Variable should be explicitly specified}"

NEUTRON_SIDE_PORT_ID="${NEUTRON_SIDE_PORT_ID:?Variable should explicitly specified}"
ICQ_UPDATE_PERIOD="${ICQ_UPDATE_PERIOD:?Variable should explicitly specified}"
SALT="${SALT:?Variable should explicitly specified}"
SUBDENOM="${SUBDENOM:?Variable should explicitly specified}"
TOKEN_METADATA_DESCRIPTION="${TOKEN_METADATA_DESCRIPTION:?Variable should explicitly specified}"
TOKEN_METADATA_DISPLAY="${TOKEN_METADATA_DISPLAY:?Variable should explicitly specified}"
TOKEN_METADATA_EXPONENT="${TOKEN_METADATA_EXPONENT:?Variable should explicitly specified}"
TOKEN_METADATA_NAME="${TOKEN_METADATA_NAME:?Variable should explicitly specified}"
TOKEN_METADATA_SYMBOL="${TOKEN_METADATA_SYMBOL:?Variable should explicitly specified}"
CORE_PARAMS_IDLE_MIN_INTERVAL="${CORE_PARAMS_IDLE_MIN_INTERVAL:?Variable should explicitly specified}"
CORE_PARAMS_LSM_REDEEM_THRESHOLD="${CORE_PARAMS_LSM_REDEEM_THRESHOLD:?Variable should explicitly specified}"
CORE_PARAMS_LSM_MIN_BOND_AMOUNT="${CORE_PARAMS_LSM_MIN_BOND_AMOUNT:?Variable should explicitly specified}"
CORE_PARAMS_LSM_REDEEM_MAX_INTERVAL="${CORE_PARAMS_LSM_REDEEM_MAX_INTERVAL:?Variable should explicitly specified}"
CORE_PARAMS_BOND_LIMIT="${CORE_PARAMS_BOND_LIMIT:?Variable should explicitly specified}"
CORE_PARAMS_MIN_STAKE_AMOUNT="${CORE_PARAMS_MIN_STAKE_AMOUNT:?Variable should explicitly specified}"
CORE_PARAMS_ICQ_UPDATE_DELAY="${CORE_PARAMS_ICQ_UPDATE_DELAY:?Variable should explicitly specified}"
STAKER_PARAMS_MIN_STAKE_AMOUNT="${STAKER_PARAMS_MIN_STAKE_AMOUNT:?Variable should explicitly specified}"
STAKER_PARAMS_MIN_IBC_TRANSFER="${STAKER_PARAMS_MIN_IBC_TRANSFER:?Variable should explicitly specified}"

source ./utils.bash

echo "DEPLOY_WALLET: $DEPLOY_WALLET"
echo "NEUTRON_RPC: $NEUTRON_RPC"
echo "NEUTRON_HOME: $NEUTRON_HOME"


main() {
  set -euo pipefail
  IFS=$'\n\t'

  pre_deploy_check_code_ids
  pre_deploy_check_balance
  pre_deploy_check_ibc_connection 
  deploy_factory
  
  register_rewards_pump_ica
  print_hermes_command $rewards_pump_ica_port $rewards_pump_ica_channel
  wait_ica_address "rewards_pump" $rewards_pump_address
  rewards_pump_counterparty_channel_id=$(get_counterparty_channel_id $rewards_pump_ica_port $rewards_pump_ica_channel)

  register_puppeteer_ica
  print_hermes_command $puppeteer_ica_port $puppeteer_ica_channel
  wait_ica_address "puppeteer" $puppeteer_address
  puppeteer_counterparty_channel_id=$(get_counterparty_channel_id $puppeteer_ica_port $puppeteer_ica_channel)

  update_msg='{
    "add_bond_provider":{
      "bond_provider_address": "'"$native_bond_provider_address"'"
    }
  }'

  msg='{
    "wasm":{
      "execute":{
        "contract_addr":"'"$core_address"'",
        "msg":"'"$(echo -n "$update_msg" | jq -c '.' | base64 | tr -d "\n")"'",
        "funds": []
      }
    }
  }'

  factory_admin_execute $factory_address "$msg"
  echo "[OK] Add Native bond provider to the Core contract"

  update_msg='{
    "add_bond_provider":{
      "bond_provider_address": "'"$lsm_share_bond_provider_address"'"
    }
  }'

  msg='{
    "wasm":{
      "execute":{
        "contract_addr":"'"$core_address"'",
        "msg":"'"$(echo -n "$update_msg" | jq -c '.' | base64 | tr -d "\n")"'",
        "funds": []
      }
    }
  }'

  factory_admin_execute $factory_address "$msg"
  echo "[OK] Add LSM share bond provider to the Core contract"

  msg='{
    "validator_set": {
      "update_validators": {
        "validators": '"$INITIAL_VALIDATORS"'
      }
    }
  }'

  factory_proxy_execute $factory_address "$msg" 1000000untrn
  echo "[OK] Add initial validators to factory"

  deploy_pump
  register_pump_ica
  print_hermes_command $pump_ica_port $pump_ica_channel
  wait_ica_address "pump" $pump_address
  pump_counterparty_channel_id=$(get_counterparty_channel_id $pump_ica_port $pump_ica_channel)

  msg='{
    "update_config":{
      "core":{
        "pump_ica_address":"'"$pump_ica_address"'"
      }
    }
  }'
  neutrond tx wasm execute "$factory_address" "$msg" --from "$DEPLOY_WALLET" "${ntx[@]}" | wait_ntx | assert_success
  echo "[OK] Add pump ICA address to Core config"

  echo
  echo   "CONTRACTS INSTANTIATION SUCCEDED"
  echo
  printf 'export FACTORY_ADDRESS="%s"\n' "$factory_address"
  printf 'export IBC_DENOM="%s"\n' "$uatom_on_neutron_denom"
  echo
  echo   "[[chains]]"
  printf 'id = "%s"\n' "NEUTRON_CHAIN_ID"
  echo   "[chains.packet_filter]"
  echo   "list = ["
  echo   "  ['$puppeteer_ica_port', '$puppeteer_ica_channel'],"
  echo   "  ['$pump_ica_port', '$pump_ica_channel'],"
  echo   "]"
  echo
  echo   "[[chains]]"
  printf 'id = "%s"\n' "$TARGET_CHAIN_ID"
  echo   "[chains.packet_filter]"
  echo   "list = ["
  echo   "  ['icahost', '$puppeteer_counterparty_channel_id'],"
  echo   "  ['icahost', '$pump_counterparty_channel_id'],"
  echo   "]"
  
}

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
