#!/bin/bash
echo "Building initia-test docker image"
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
git clone https://github.com/initia-labs/initia -b v0.2.15

# Copy Dockerfile to initia directory for arm and Dockerfile.x86 for x86
if [[ "$(arch)" == "arm64" ]]; then
    cp ./Dockerfile ./initia/Dockerfile
else
    echo "Building for x86"
    cp ./Dockerfile.x86 ./initia/Dockerfile
fi

if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    echo ""
    VERSION=":$VERSION"
fi
docker build initia -t ${ORG}initia-test${VERSION}
rm -rf ./initia