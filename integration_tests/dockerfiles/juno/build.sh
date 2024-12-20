#!/bin/bash
DIR="$(dirname $0)"
COMMIT_HASH_OR_BRANCH="v25.0.0"
cd $DIR
VERSION=$(cat ../../package.json | jq -r '.version')
if [[ "$CI" == "true" ]]; then
    VERSION="_$VERSION"
    ORG=neutronorg/lionco-contracts:
else
    VERSION=":$VERSION"
fi
git clone git@github.com:CosmosContracts/juno.git

cd juno
git checkout $COMMIT_HASH_OR_BRANCH
sed -i '' '/\/cosmos.staking.v1beta1.Query\/Pool/a\
"/cosmos.staking.v1beta1.Query/DelegatorDelegations":          &stakingtypes.QueryDelegatorDelegationsResponse{},\
"/cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations": &stakingtypes.QueryDelegatorUnbondingDelegationsResponse{},
' ./app/keepers/keepers.go
cd ..

docker build juno -t ${ORG}juno-test${VERSION}
rm -rf ./juno
