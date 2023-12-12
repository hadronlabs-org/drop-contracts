#!/bin/bash

declare -A ADDR_TO_NAME_MAP

sleep 10 #trying to deal with runtime error: invalid memory address or nil pointer dereference: panic

while IFS=" " read -r ADDR NAME; do
    TRIMMED_ADDR=${ADDR:0:-6}
    ADDR_TO_NAME_MAP["$TRIMMED_ADDR"]=$NAME
done < <(gaiad keys list --keyring-backend=test --home=/opt --output json | jq -r '.[] | .address + " " + .name')

gaiad query staking validators --output json | jq -r '.validators | .[] | .operator_address' | while read -r VAL_ADDRESS; do
    KEY_ADDRESS="cosmos${VAL_ADDRESS:13:-6}"

    KEY_NAME=${ADDR_TO_NAME_MAP["$KEY_ADDRESS"]}

    if [ -n "$KEY_NAME" ]; then
        gaiad tx staking validator-bond "$VAL_ADDRESS" --from "$KEY_NAME" --chain-id testgaia --home=/opt --keyring-backend=test --broadcast-mode=block -y >> /opt/gaiad.log 2>&1  
    else
        echo "No key name found for address: $KEY_ADDRESS"
    fi
done