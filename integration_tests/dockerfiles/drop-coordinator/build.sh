#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
cd ../../../
VERSION=$(cat integration_tests/package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
docker build -f integration_tests/dockerfiles/drop-coordinator/Dockerfile -t ${ORG}neutron-query-relayer-cli-test${VERSION} . 
