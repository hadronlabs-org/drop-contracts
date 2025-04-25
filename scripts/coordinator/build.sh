#!/bin/bash
DIR="$(dirname $0)"
cd $DIR

DOCKER_DATA_DIR="docker_data"

mkdir $DOCKER_DATA_DIR

echo "Copying TS client ..."
rsync -av --exclude=node_modules ../../ts-client/ $DOCKER_DATA_DIR/ts-client

echo "Copying Query Relayer CLI ..."
QUERY_RELAYER_COMMIT_HASH_OR_BRANCH="main"
git clone https://github.com/hadronlabs-org/neutron-query-relayer-cli.git $DOCKER_DATA_DIR/neutron-query-relayer-cli
cd $DOCKER_DATA_DIR/neutron-query-relayer-cli
git checkout $QUERY_RELAYER_COMMIT_HASH_OR_BRANCH
cd ../..

echo "Copying Neutron ..."
NEUTORN_COMMIT_HASH_OR_BRANCH="v5.1.2"
git clone https://github.com/neutron-org/neutron.git $DOCKER_DATA_DIR/neutron
cd $DOCKER_DATA_DIR/neutron
git checkout $NEUTORN_COMMIT_HASH_OR_BRANCH
cd ../..

echo "Copying coordinator ..."
rsync -av --exclude=${DOCKER_DATA_DIR} --exclude=node_modules ./ ./$DOCKER_DATA_DIR/coordinator

# QR_COMMIT=$(git log -1 --format='%H')
# QR_VERSION=$(git describe --tags | sed 's/^v//')

# LD_FLAGS="-X github.com/hadronlabs-org/neutron-query-relayer-cli/internal/app.Version=$QR_VERSION -X github.com/hadronlabs-org/neutron-query-relayer-cli/internal/app.Commit=$QR_COMMIT"

BUILDING_ARCHS="linux/arm64"
if [ "$(uname -m)" = "x86_64" ]; then
  BUILDING_ARCHS="$BUILDING_ARCH,linux/amd64"
fi
# --platform linux/amd64,linux/arm64
docker build --platform $BUILDING_ARCHS -t dropprotocol/coordinator .
rm -rf $DOCKER_DATA_DIR