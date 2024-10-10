#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
git clone https://github.com/cosmos/gaia.git -b v19.2.0
cp ./Dockerfile ./gaia
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
docker build gaia -t ${ORG}gaia-test${VERSION}
rm -rf ./gaia