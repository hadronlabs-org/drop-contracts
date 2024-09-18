#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
git clone https://github.com/initia-labs/initia -b v0.2.15
cp ./Dockerfile ./initia
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    echo ""
    VERSION=":$VERSION"
fi
docker build initia -t ${ORG}initia-test${VERSION}
rm -rf ./initia