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
  factory_contract_address=$(get_contract_address $factory_code_id $deploy_wallet $SALT)
  echo "Factory address: $factory_contract_address"
  core_contract_contract=$(get_contract_address $core_code_id $factory_contract_address $SALT)
  echo "Core address: $core_contract_contract"
  puppeteer_contract_address=$(get_contract_address $puppeteer_code_id $deploy_wallet $SALT)
  echo "Puppeteer address: $puppeteer_contract_address"
  strategy_contract_address=$(get_contract_address $strategy_code_id $factory_contract_address $SALT)
  echo "Strategy address: $strategy_contract_address"
  validators_set_contract_address=$(get_contract_address $validators_set_code_id $factory_contract_address $SALT)
  echo "Validators set address: $validators_set_contract_address"
  lsm_share_bond_provider_contract_address=$(get_contract_address $lsm_share_bond_provider_code_id $deploy_wallet $SALT)
  echo "LSM share bond provider address: $lsm_share_bond_provider_contract_address"
  withdrawal_manager_contract_address=$(get_contract_address $withdrawal_manager_code_id $factory_contract_address $SALT)
  echo "Withdrawal manager address: $withdrawal_manager_contract_address"
  splitter_contract_address=$(get_contract_address $splitter_code_id $factory_contract_address $SALT)
  echo "Splitter address: $splitter_contract_address"
  

  uatom_on_neutron_denom="ibc/$(printf 'transfer/%s/%s' "$NEUTRON_SIDE_TRANSFER_CHANNEL_ID" "$TARGET_BASE_DENOM" \
    | sha256sum - | awk '{print $1}' | tr '[:lower:]' '[:upper:]')"
  echo "[OK] IBC denom of $TARGET_BASE_DENOM on Neutron is $uatom_on_neutron_denom"

  native_bond_provider_contract_address=$(deploy_native_bond_provider "$factory_contract_address" "$core_contract_contract" "$puppeteer_contract_address" "$strategy_contract_address")
  echo "[OK] Native bond provider address: $native_bond_provider_contract_address"

  deployed_lsm_share_bond_provider_contract_address=$(deploy_lsm_share_bond_provider "$factory_contract_address" "$core_contract_contract" "$puppeteer_contract_address" "$validators_set_contract_address")
  echo "[OK] Deployed lsm share bond provider address: $deployed_lsm_share_bond_provider_contract_address"

  deployed_puppeteer_contract_address=$(deploy_puppeteer "$factory_contract_address" "$core_contract_contract" "$native_bond_provider_contract_address" "$lsm_share_bond_provider_contract_address")
  echo "[OK] Deployed puppeteer address: $deployed_puppeteer_contract_address"

  unbonding_pump_contract_address=$(deploy_pump "drop-unbonding-pump" "$factory_contract_address" "$withdrawal_manager_contract_address")
  echo "[OK] Deployed unbonding pump address: $unbonding_pump_contract_address"

  rewards_pump_contract_address=$(deploy_pump "drop-rewards-pump" "$factory_contract_address" "$splitter_contract_address")
  echo "[OK] Deployed rewards pump address: $rewards_pump_contract_address"
  
  pre_instantiated_contracts='{
    "native_bond_provider_address":"'"$native_bond_provider_contract_address"'",
    "puppeteer_address":"'"$puppeteer_contract_address"'",
    "lsm_share_bond_provider_address":"'"$lsm_share_bond_provider_contract_address"'",
    "unbonding_pump_address":"'"$unbonding_pump_contract_address"'",
    "rewards_pump_address":"'"$rewards_pump_contract_address"'"
  }'

  bond_providers='[
    {"name":"native_bond_provider","contract_address":"'"$native_bond_provider_contract_address"'"},
    {"name":"lsm_share_bond_provider","contract_address":"'"$lsm_share_bond_provider_contract_address"'"}
  ]'

  pumps='[
    {"name":"unbonding_pump","contract_address":"'"$unbonding_pump_contract_address"'"},
    {"name":"rewards_pump","contract_address":"'"$rewards_pump_contract_address"'"}
  ]'
  
  deploy_factory "$pre_instantiated_contracts" "$bond_providers" "$pumps"

  top_up_address "$puppeteer_contract_address"
  
  register_ica "rewards_pump" "$rewards_pump_contract_address"
  print_hermes_command $rewards_pump_ica_port $rewards_pump_ica_channel
  wait_ica_address "rewards_pump" $unbonding_pump_contract_address
  rewards_pump_counterparty_channel_id=$(get_counterparty_channel_id $rewards_pump_ica_port $rewards_pump_ica_channel)

  register_ica "puppeteer" "$puppeteer_contract_address"
  print_hermes_command $puppeteer_ica_port $puppeteer_ica_channel
  wait_ica_address "puppeteer" $puppeteer_contract_address
  puppeteer_counterparty_channel_id=$(get_counterparty_channel_id $puppeteer_ica_port $puppeteer_ica_channel)

  update_msg='{
    "add_bond_provider":{
      "bond_provider_address": "'"$native_bond_provider_contract_address"'"
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

  factory_admin_execute "$factory_address" "$msg" 250000untrn
  echo "[OK] Add Native bond provider to the Core contract"

  update_msg='{
    "add_bond_provider":{
      "bond_provider_address": "'"$lsm_share_bond_provider_contract_address"'"
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

  factory_admin_execute "$factory_address" "$msg" 250000untrn
  echo "[OK] Add LSM share bond provider to the Core contract"


  REWARDS_ADDRESS=${REWARDS_ADDRESS:-$rewards_pump_ica_address}
  update_msg='{
   "setup_protocol": {
      "rewards_withdraw_address": "'"$REWARDS_ADDRESS"'"
    }
  }'

  msg='{
    "wasm":{
      "execute":{
        "contract_addr":"'"$puppeteer_contract_address"'",
        "msg":"'"$(echo -n "$update_msg" | jq -c '.' | base64 | tr -d "\n")"'",
        "funds": [
          {
            "amount": "200000",
            "denom": "untrn"
          }
        ]
      }
    }
  }'

  factory_admin_execute "$factory_address" "$msg" 250000untrn
  echo "[OK] Add Rewards withdraw address to Puppeteer ICA"

  msg='{
    "validator_set": {
      "update_validators": {
        "validators": '"$INITIAL_VALIDATORS"'
      }
    }
  }'

  factory_proxy_execute "$factory_address" "$msg" 3000000untrn
  echo "[OK] Add initial validators to factory"

  register_ica "pump" "$unbonding_pump_contract_address"
  print_hermes_command $pump_ica_port $pump_ica_channel
  wait_ica_address "pump" $unbonding_pump_contract_address
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
