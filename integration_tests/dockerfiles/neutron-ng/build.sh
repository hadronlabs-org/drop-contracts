#!/bin/bash
DIR="$(dirname $0)"
COMMIT_HASH_OR_BRANCH="test/move-to-sovereign"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
git clone git@github.com:neutron-org/neutron-private.git
cd neutron-private
git checkout $COMMIT_HASH_OR_BRANCH
docker buildx build --load --build-context app=. -t ${ORG}neutron-ng-test${VERSION} --build-arg BINARY=neutrond .
cd ..
rm -rf ./neutron-private