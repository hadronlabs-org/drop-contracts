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
git clone git@github.com:neutron-org/neutron-private
cd neutron-private
git checkout $COMMIT_HASH_OR_BRANCH
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' 's/Ym+xqrhq1kJzyyJICGc2gB43+EETGy72sumZUU1wCt8=/EIdcrUrrPt9UgCUZTupM28XoMPHNsLvMDHRD0oKe\/Ck=/g' go.sum
else
    sed -i 's/Ym+xqrhq1kJzyyJICGc2gB43+EETGy72sumZUU1wCt8=/EIdcrUrrPt9UgCUZTupM28XoMPHNsLvMDHRD0oKe\/Ck=/g' go.sum
fi
grep 'EIdcrUrrPt9UgCUZTupM28XoMPHNsLvMDHRD0oKe/Ck=' go.sum || exit 1
go mod tidy
docker buildx build --load --build-context app=. -t ${ORG}neutronv2-test${VERSION} --build-arg BINARY=neutrond .
cd ..
rm -rf ./neutron-private
