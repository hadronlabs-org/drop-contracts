#!/bin/bash

if [[ -z "$COORDINATOR_NATIVE_MODE" || "$COORDINATOR_NATIVE_MODE" == "false" ]]
then
    echo "Creating query relayer wallet ..."
    echo "$RELAYER_WALLET_MNEMONIC" | neutrond keys add $RELAYER_NEUTRON_CHAIN_SIGN_KEY_NAME --recover --keyring-backend $RELAYER_NEUTRON_CHAIN_KEYRING_BACKEND 
fi

echo "Starting coordinator..."

while true; do
    yarn ts-node src/service.ts
    echo "Restarting coordinator after crash..."
    sleep 30
done