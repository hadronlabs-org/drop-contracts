#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/
else
    VERSION=":$VERSION"
fi
docker build . -t ${ORG}hermes-test${VERSION}

