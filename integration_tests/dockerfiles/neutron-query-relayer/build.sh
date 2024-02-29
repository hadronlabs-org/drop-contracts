#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
git clone -b foxpy/low-submission-margin-period https://github.com/neutron-org/neutron-query-relayer
VERSION=$(cat ../../package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
cd neutron-query-relayer
GVERSION=$(echo $(git describe --tags) | sed 's/^v//')
COMMIT=$(git log -1 --format='%H')
ldflags="-X github.com/neutron-org/neutron-query-relayer/internal/app.Version=$GVERSION -X github.com/neutron-org/neutron-query-relayer/internal/app.Commit=$COMMIT" 
docker build --build-arg LDFLAGS="$ldflags" . -t ${ORG}neutron-query-relayer-test${VERSION}
cd ..
rm -rf ./neutron-query-relayer
