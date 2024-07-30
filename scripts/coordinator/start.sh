#!/bin/bash

echo "Starting coordinator..."

while true; do
    dotenvx run -f $1 -- ts-node src/service.ts
    echo "Restarting coordinator after crash..."
    sleep 1
done