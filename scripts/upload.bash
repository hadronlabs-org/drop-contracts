#!/usr/bin/env bash

NEUTRON_RPC="${NEUTRON_RPC:-tcp://0.0.0.0:26657}"
NEUTRON_HOME="${NEUTRON_HOME:-$HOME/.neutrond}"
NEUTRON_CHAIN_ID="${NEUTRON_CHAIN_ID:-test-1}"
GAS_PRICES="${GAS_PRICES:-0.005}"
KEYRING_BACKEND="${KEYRING_BACKEND:-test}"
DEPLOY_WALLET="${DEPLOY_WALLET:-demowallet1}"
MIN_NTRN_REQUIRED="${MIN_NTRN_REQUIRED:-10}"

source ./utils.bash

echo "DEPLOY_WALLET: $DEPLOY_WALLET"
echo "NEUTRON_RPC: $NEUTRON_RPC"


main() {
  set -euo pipefail
  IFS=$'\n\t'

  pre_deploy_check_balance
  deploy_wasm_code

  echo
  echo   "CONTRACTS UPLOAD SUCCEDED"
  echo
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
