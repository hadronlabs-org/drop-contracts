#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
cp ../../../../neutron-query-relayer/ ./neutron-query-relayer-cli -r
# git clone https://github.com/hadronlabs-org/neutron-query-relayer-cli
VERSION=$(cat ../../package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
cd neutron-query-relayer-cli
GVERSION=$(echo $(git describe --tags) | sed 's/^v//')
COMMIT=$(git log -1 --format='%H')
ldflags="-X github.com/hadronlabs-org/neutron-query-relayer-cli/internal/app.Version=$GVERSION -X github.com/hadronlabs-org/neutron-query-relayer-cli/internal/app.Commit=$COMMIT" 
docker build --build-arg LDFLAGS="$ldflags" . -t ${ORG}neutron-query-relayer-cli-test${VERSION}
cd ..
rm -rf ./neutron-query-relayer-cli