#!/bin/bash
DIR="$(dirname $0)"
cd $DIR
git clone https://github.com/neutron-org/neutron-query-relayer
cd neutron-query-relayer
make build-docker
cd ..
rm -rf ./neutron-query-relayer