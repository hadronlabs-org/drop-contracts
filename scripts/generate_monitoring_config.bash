#!/usr/bin/env bash

FACTORY_ADDRESS="${FACTORY_ADDRESS:-$1}"
NEUTRON_RPC="${NEUTRON_RPC:-$2}"
NEUTRON_RPC="${NEUTRON_RPC:-tcp://0.0.0.0:26657}"

nq=()
source ./utils.bash

main() {
  set -euo pipefail
  IFS=$'\n\t'

  if [[ $# -eq 0 ]]; then
    die "FACTORY_ADDRESS is not defined"
  fi

  core_contract="$(neutrond query wasm contract-state smart "$FACTORY_ADDRESS" '{"state":{}}' "${nq[@]}" | jq -r '.data.core_contract')" && echo -n '.'
  puppeteer_contract="$(neutrond query wasm contract-state smart "$FACTORY_ADDRESS" '{"state":{}}' "${nq[@]}" | jq -r '.data.puppeteer_contract')" && echo -n '.'
  staker_contract="$(neutrond query wasm contract-state smart "$FACTORY_ADDRESS" '{"state":{}}' "${nq[@]}" | jq -r '.data.staker_contract')" && echo -n '.'
  token_contract="$(neutrond query wasm contract-state smart "$FACTORY_ADDRESS" '{"state":{}}' "${nq[@]}" | jq -r '.data.token_contract')" && echo -n '.'
  admin="$(neutrond query wasm contract "$FACTORY_ADDRESS" "${nq[@]}" | jq -r '.contract_info.admin')" && echo -n '.'
  base_ibc_denom="$(neutrond query wasm contract-state smart "$core_contract" '{"config":{}}' "${nq[@]}" | jq -r '.data.base_denom')" && echo -n '.'
  base_denom="$(neutrond query ibc-transfer denom-trace "$base_ibc_denom" "${nq[@]}" | jq -r '.denom_trace.base_denom')" && echo -n '.'
  puppeteer_ica="$(neutrond query wasm contract-state smart "$puppeteer_contract" '{"ica":{}}' "${nq[@]}" | jq -r '.data.registered.ica_address')" && echo -n '.'
  staker_ica="$(neutrond query wasm contract-state smart "$staker_contract" '{"ica":{}}' "${nq[@]}" | jq -r '.data.registered.ica_address')" && echo -n '.'
  pump_ica="$(neutrond query wasm contract-state smart "$core_contract" '{"config":{}}' "${nq[@]}" | jq -r '.data.pump_ica_address')" && echo -n '.'
  validators_set_contract="$(neutrond query wasm contract-state smart "$FACTORY_ADDRESS" '{"state":{}}' "${nq[@]}" | jq -r '.data.validators_set_contract')" && echo -n '.'
  watched_validators=""
  for validator in $(neutrond query wasm contract-state smart "$validators_set_contract" '{"validators":{}}' "${nq[@]}" | jq -r '.data[] | .valoper_address'); do
    watched_validators="${watched_validators}- ${validator}"$'\n'
     echo -n '.'
  done
  echo

  config="$(cat << EOF
clients_timeout_seconds: 20
export_period_seconds: 20
logger_level: info
listen_prometheus: 9998

chain_registry:
  provider: provider
  apis:
    neutron:
      rpc: %s
      rest: <FIll NEUTRON REST MANUALLY>
    provider:
      rpc: <FIll PROVIDER RPC MANUALLY>
      rest: <FIll PROVIDER REST MANUALLY>

factory_address: %s
core_address: %s
puppeteer_address: %s
staker_address: %s

ld_token: factory/%s/drop

watched_wallets:
  - name: testnet_deployment_owner
    chain_name: neutron
    address: %s
    denoms:
    - untrn
    - %s
  - name: testnet_deployment_owner
    chain_name: provider
    address: <FILL %s on PROVIDER CHAIN MANUALLY>
    denoms:
    - %s
  - name: core_contract
    chain_name: neutron
    address: %s
    denoms:
    - %s
  - name: puppeteer_contract
    chain_name: neutron
    address: %s
    denoms:
    - %s
  - name: puppeteer_ica
    chain_name: provider
    address: %s
    denoms:
    - %s
  - name: staker_contract
    chain_name: neutron
    address: %s
    denoms:
    - %s
  - name: staker_ica
    chain_name: provider
    address: %s
    denoms:
    - %s
  - name: pump_ica
    chain_name: provider
    address: %s
    denoms:
    - %s
watched_delegator: %s
watched_validators:
%s
EOF
  )"

  echo "Manually complete this config and use it for metrics exporter:"
  echo
  echo
  # shellcheck disable=SC2059
  printf "${config}\n"    \
    "$NEUTRON_RPC"        \
    "$FACTORY_ADDRESS"    \
    "$core_contract"      \
    "$puppeteer_contract" \
    "$staker_contract"    \
    "$token_contract"     \
    "$admin"              \
    "$base_ibc_denom"     \
    "$admin"              \
    "$base_denom"         \
    "$core_contract"      \
    "$base_ibc_denom"     \
    "$puppeteer_contract" \
    "$base_ibc_denom"     \
    "$puppeteer_ica"      \
    "$base_denom"         \
    "$staker_contract"    \
    "$base_ibc_denom"     \
    "$staker_ica"         \
    "$base_denom"         \
    "$pump_ica"           \
    "$base_denom"         \
    "$puppeteer_ica"      \
    "$watched_validators"
}

exec 3>&1
error_output="$(main "$@" 2>&1 1>&3)"
exit_code=$?
exec 3>&-

if [[ ! $exit_code -eq 0 ]]; then
  echo
  echo "MONITORING CONFIG GENERATION FAILED WITH CODE $exit_code"
  echo "Error output:"
  echo "$error_output"
fi

exit $exit_code
