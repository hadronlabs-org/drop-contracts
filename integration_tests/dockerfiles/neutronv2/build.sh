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
echo "1"
echo "$OSTYPE"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "darwin"
    sed -i '' 's/Ym+xqrhq1kJzyyJICGc2gB43+EETGy72sumZUU1wCt8=/EIdcrUrrPt9UgCUZTupM28XoMPHNsLvMDHRD0oKe\/Ck=/g' go.sum
else
    echo "linux"
    sed -i 's/Ym+xqrhq1kJzyyJICGc2gB43+EETGy72sumZUU1wCt8=/EIdcrUrrPt9UgCUZTupM28XoMPHNsLvMDHRD0oKe\/Ck=/g' go.sum
fi
echo "2"
grep 'EIdcrUrrPt9UgCUZTupM28XoMPHNsLvMDHRD0oKe/Ck=' go.sum || exit 1
echo "3"
go mod tidy
echo "4"
docker buildx build --load --build-context app=. -t ${ORG}neutronv2-test${VERSION} --build-arg BINARY=neutrond .
cd ..
rm -rf ./neutron-private
