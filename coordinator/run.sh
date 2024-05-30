#!/bin/sh
NODE=${NODE:-node}
echo "Waiting for a first block..."
while ! curl -f ${NODE}:1317/cosmos/base/tendermint/v1beta1/blocks/1 >/dev/null 2>&1; do
  sleep 1
done

if [[ "$MANUAL_MODE" == "true" ]]; then
  while true; do
  :
  done
else
  echo "Start relayer coordinator"

  yarn dev
fi



