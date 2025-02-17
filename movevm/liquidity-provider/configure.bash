#!/usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'
cd "$(dirname "$0")"

if [ "$#" -ne 4 ]; then
  echo "Usage: $0 <pair address> <asset address> <owner address> <recipient address>" >&2
  exit 1
fi
pair_address="$1"
asset_address="$2"
owner_address="$3"
recipient_address="$4"

sed -i "s/^pair \= \".*\"/pair = \"$pair_address\"/g" Move.toml
sed -i "s/^asset \= \".*\"/asset = \"$asset_address\"/g" Move.toml
sed -i "s/^me \= \".*\"/me = \"$owner_address\"/g" Move.toml
sed -i "s/^recipient \= \".*\"/recipient = \"$recipient_address\"/g" Move.toml
