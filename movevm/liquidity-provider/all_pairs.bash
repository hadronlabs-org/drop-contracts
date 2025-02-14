#!/usr/bin/env bash

# Fetch all DEX pairs on initiation-2 and print their info

set -euo pipefail
IFS=$'\n\t'

declare -a q=(
	"--output" "json"
	"--node" "https://rpc.initiation-2.initia.xyz:443"
)

all="$(initiad query move view 0x1 dex get_all_pairs --args '["option<address>:null", "option<address>:null", "option<address>:null", "u8:255"]' "${q[@]}" | jq -r '.data')"
for pair in $(echo "$all" | jq -rc '.[]'); do
	lp="$(echo "$pair" | jq -r '.liquidity_token')"
	coin_a="$(echo "$pair" | jq -r '.coin_a')"
	coin_b="$(echo "$pair" | jq -r '.coin_b')"
	metadata="$(initiad query move resource "$lp" 0x1::fungible_asset::Metadata "${q[@]}")"
	name="$(echo "$metadata" | jq -r '.resource.move_resource' | jq '.data.name')"
	symbol="$(echo "$metadata" | jq -r '.resource.move_resource' | jq '.data.symbol')"
	supply="$(initiad query move resource "$lp" 0x1::fungible_asset::Supply "${q[@]}")"
	supply="$(echo "$supply" | jq -r '.resource.move_resource' | jq '.data.current')"
	echo "LP: $lp, name: $name, symbol: $symbol, supply: $supply, coin_a: $coin_a, coin_b: $coin_b"
done
