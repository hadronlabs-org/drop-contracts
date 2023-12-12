#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
git clone https://github.com/iqlusioninc/liquidity-staking-module -b sam/simapp-enable-ibc
cp ./Dockerfile ./liquidity-staking-module
docker build liquidity-staking-module -t lsm
rm -rf ./liquidity-staking-module