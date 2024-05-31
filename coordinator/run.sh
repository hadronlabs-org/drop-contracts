#!/bin/sh
NODE=${NODE:-node}
echo "Waiting for a first block..."
while ! curl -f ${NODE}:1317/cosmos/base/tendermint/v1beta1/blocks/1 >/dev/null 2>&1; do
  sleep 1
done

echo "Manual mode: $MANUAL_MODE"
if [ "$MANUAL_MODE" = "true" ]; then
  while true; do
    echo "Running query relayer in manual mode..."
    sleep 10
  done
else
  echo "Start relayer coordinator"

  yarn dev
fi
