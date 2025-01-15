#!/usr/bin/env bash

NEUTRON_RPC="${NEUTRON_RPC:-tcp://0.0.0.0:26657}"
NEUTRON_HOME="${NEUTRON_HOME:-$HOME/.neutrond}"
NEUTRON_CHAIN_ID="${NEUTRON_CHAIN_ID:-test-1}"
GAS_PRICES="${GAS_PRICES:-0.005}"
KEYRING_BACKEND="${KEYRING_BACKEND:-test}"
DEPLOY_WALLET="${DEPLOY_WALLET:-demowallet1}"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-../../artifacts}"

CONTRACT_ADDRESS="${CONTRACT_ADDRESS:-$1}"
CONTRACT_NAME="${CONTRACT_NAME:-$2}"
CODE_ID=$3

source ./utils.bash


main() {
  set -euo pipefail
  IFS=$'\n\t'

  if [[ -z $CODE_ID ]]; then
    store_code "$CONTRACT_NAME"
  else
    declare -g "${CONTRACT_NAME}_code_id=$CODE_ID"
  fi

  code_id="${CONTRACT_NAME}_code_id"  
  echo "[OK] Contract uploaded successfully. Code ID: ${!code_id}"

  migrate_msg='{}'

  migrate_contract $CONTRACT_ADDRESS ${!code_id} "$migrate_msg"
  echo "[OK] Contract migrated"  
}

exec 3>&1
error_output="$(main 2>&1 1>&3)"
exit_code=$?
exec 3>&-

if [[ ! $exit_code -eq 0 ]]; then
  echo
  echo "MIGRATION FAILED WITH CODE $exit_code"
  echo "Error output:"
  echo "$error_output"
fi

exit $exit_code
