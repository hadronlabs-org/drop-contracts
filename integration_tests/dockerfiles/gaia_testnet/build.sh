#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
git clone https://github.com/cosmos/gaia.git -b v15.0.0
cp ./Dockerfile ./gaia

docker build gaia -t ${ORG}gaia_testnet-test${VERSION}
rm -rf ./gaia