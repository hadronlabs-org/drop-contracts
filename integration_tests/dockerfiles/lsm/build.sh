#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
git clone https://github.com/iqlusioninc/liquidity-staking-module -b sam/simapp-enable-ibc
cp ./Dockerfile ./liquidity-staking-module
docker build liquidity-staking-module -t lsm-test:$VERSION
rm -rf ./liquidity-staking-module