#!/bin/bash
DIR="$(dirname $0)"
COMMIT_HASH_OR_BRANCH="main"
cd $DIR
git clone https://github.com/neutron-org/neutron
cd neutron
git checkout $COMMIT_HASH_OR_BRANCH
make build-docker-image
cd ..
rm -rf ./neutron