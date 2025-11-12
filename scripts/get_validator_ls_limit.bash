#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <rest_api> <validator_address>" >&2
  exit 1
fi

rest="$1"
validator="$2"

staking_params="$(curl -s "$rest/cosmos/staking/v1beta1/params" | jq -r '.params')"
bond_factor="$(echo "$staking_params" | jq -r '.validator_bond_factor')"

staking_info="$(curl -s "$rest/cosmos/staking/v1beta1/validators/$validator" | jq -r '.validator')"
delegator_shares="$(echo "$staking_info" | jq -r '.delegator_shares')"
tokens="$(echo "$staking_info" | jq -r '.tokens')"
bond_shares="$(echo "$staking_info" | jq -r '.validator_bond_shares')"
liquid_shares="$(echo "$staking_info" | jq -r '.liquid_shares')"

ls_limit="$(python3 -c "print($delegator_shares / $tokens * ($bond_shares * $bond_factor - $liquid_shares) / 1e6, end='')")"
echo "$ls_limit"
