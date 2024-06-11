#!/bin/sh
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
git clone https://github.com/iqlusioninc/liquidity-staking-module -b sam/simapp-enable-ibc
cp ./Dockerfile ./liquidity-staking-module
docker build liquidity-staking-module -t ${ORG}lsm-test${VERSION}
rm -rf ./liquidity-staking-module