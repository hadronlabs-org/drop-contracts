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
ARCH=$(uname -m)

if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  DOCKERFILE="Dockerfile.aarch64"
else
  DOCKERFILE="Dockerfile.x86_64"
fi
docker build -f $DOCKERFILE -t ${ORG}hermes-test${VERSION} .


