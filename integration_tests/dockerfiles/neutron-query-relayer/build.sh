#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
git clone https://github.com/neutron-org/neutron-query-relayer
VERSION=$(cat ../../package.json | jq -r '.version')
cd neutron-query-relayer
GVERSION=$(echo $(git describe --tags) | sed 's/^v//')
COMMIT=$(git log -1 --format='%H')
ldflags="-X github.com/neutron-org/neutron-query-relayer/internal/app.Version=$VERSION -X github.com/neutron-org/neutron-query-relayer/internal/app.Commit=$COMMIT" 
docker build --build-arg LDFLAGS="$ldflags" . -t neutron-query-relayer-test:$VERSION
cd ..
rm -rf ./neutron-query-relayer